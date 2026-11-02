//! Native collection snapshots and bounded UUID-linked planning progress.
//! Reporting completion never fabricates a game observation or resource spend.
use crate::individual_details::DetailsView;
use crate::inventory::{Error, InventoryRepository, Pokemon, PokemonId, Result, Tag};
use crate::reference_store::digest;
use crate::strategy::{
    self, CpMultiplier, Mechanics, StrategyRecommendation, StrategyReport, StrategyStep,
};
use rusqlite::{params, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MAX_REPORT_BYTES: usize = 128 * 1024;
const MAX_PREVIOUS_BYTES: usize = 256 * 1024;
const MAX_PROGRESS_BYTES: usize = 16 * 1024;
const PREFERENCE_KEY: &str = "strategy-preferences";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GoalContext {
    pub goal: String,
    pub stardust_budget: Option<u32>,
    pub revision: String,
}

impl Default for GoalContext {
    fn default() -> Self {
        Self {
            goal: "balanced".into(),
            stardust_budget: None,
            revision: "0".into(),
        }
    }
}

impl GoalContext {
    fn validate(&self) -> Result<()> {
        if !matches!(
            self.goal.as_str(),
            "balanced"
                | "collection"
                | "gym_attack"
                | "gym_defense"
                | "rocket"
                | "great"
                | "ultra"
                | "master"
        ) || parse_revision(&self.revision).is_err()
        {
            return Err(Error::InvalidInput(
                "Invalid strategy goal or preference revision",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionStrategy {
    pub context: String,
    pub reports: Vec<StrategyReport>,
    pub preferences: GoalContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreviousRoadmap {
    pub context: String,
    pub report: StrategyReport,
    pub progress: BTreeMap<String, String>,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapView {
    pub context: String,
    pub current_context: String,
    pub revision: String,
    pub report: Option<StrategyReport>,
    pub previous: Option<PreviousRoadmap>,
    pub progress: BTreeMap<String, String>,
    pub stale: bool,
    pub generated_at: Option<String>,
}

struct AnalysisInputs {
    context: String,
    collection: Vec<(Pokemon, Vec<Tag>)>,
    details: BTreeMap<String, DetailsView>,
    mechanics: Option<Mechanics>,
    preferences: GoalContext,
    package: Option<crate::reference_store::LocalPackage>,
    resources: BTreeMap<String, crate::preparation_resources::ResourceBudget>,
}

struct StoredRoadmap {
    context: String,
    report: StrategyReport,
    previous: Option<PreviousRoadmap>,
    progress: BTreeMap<String, String>,
    revision: i64,
    generated_at: String,
}

fn parse_revision(value: &str) -> Result<i64> {
    value
        .parse::<i64>()
        .ok()
        .filter(|v| *v >= 0 && v.to_string() == value)
        .ok_or(Error::InvalidInput("Invalid nonnegative revision"))
}

fn encode<T: Serialize>(value: &T, maximum: usize) -> Result<String> {
    let text = serde_json::to_string(value)
        .map_err(|_| Error::InvalidInput("Unable to encode strategy data"))?;
    if text.len() > maximum {
        return Err(Error::InvalidInput(
            "Strategy data exceeds the bounded persistence limit",
        ));
    }
    Ok(text)
}

fn decode<T: serde::de::DeserializeOwned>(text: &str, maximum: usize) -> Result<T> {
    if text.len() > maximum {
        return Err(Error::InvalidInput(
            "Stored roadmap exceeds the bounded persistence limit",
        ));
    }
    serde_json::from_str(text)
        .map_err(|_| Error::InvalidInput("Stored roadmap or preferences are corrupted"))
}

fn valid_status(status: &str) -> bool {
    matches!(
        status,
        "pending" | "postponed" | "ignored" | "review_requested" | "completed"
    )
}

fn validate_progress(report: &StrategyReport, progress: &BTreeMap<String, String>) -> Result<()> {
    if progress.len() > 128 || report.steps.len() > 128 {
        return Err(Error::InvalidInput("Too many roadmap steps"));
    }
    for (id, status) in progress {
        let step = report
            .steps
            .iter()
            .find(|step| &step.id == id)
            .ok_or(Error::InvalidInput(
                "Stored progress refers to an unknown step",
            ))?;
        if !valid_status(status)
            || status == "completed" && !matches!(step.action.as_str(), "compare" | "wait")
        {
            return Err(Error::InvalidInput(
                "Game actions and observation reviews require explicit confirmation",
            ));
        }
    }
    Ok(())
}

impl StoredRoadmap {
    fn view(self, current_context: String) -> RoadmapView {
        RoadmapView {
            stale: self.context != current_context,
            context: self.context,
            current_context,
            revision: self.revision.to_string(),
            report: Some(self.report),
            previous: self.previous,
            progress: self.progress,
            generated_at: Some(self.generated_at),
        }
    }
}

impl InventoryRepository {
    pub fn strategy_preferences(&self) -> Result<GoalContext> {
        let text: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM application_settings WHERE key=?1",
                [PREFERENCE_KEY],
                |row| row.get(0),
            )
            .optional()?;
        let result: GoalContext = match text {
            Some(text) => decode(&text, 4096)?,
            None => GoalContext::default(),
        };
        result.validate()?;
        Ok(result)
    }

    pub fn save_strategy_preferences(
        &self,
        expected_revision: i64,
        goal: &str,
        stardust_budget: Option<u32>,
    ) -> Result<GoalContext> {
        let transaction =
            Transaction::new_unchecked(&self.connection, TransactionBehavior::Immediate)?;
        let current = self.strategy_preferences()?;
        if parse_revision(&current.revision)? != expected_revision {
            return Err(Error::ObservationConflict);
        }
        if current.goal == goal && current.stardust_budget == stardust_budget {
            transaction.commit()?;
            return Ok(current);
        }
        let next = GoalContext {
            goal: goal.into(),
            stardust_budget,
            revision: expected_revision
                .checked_add(1)
                .ok_or(Error::InvalidInput("Preference revision overflow"))?
                .to_string(),
        };
        next.validate()?;
        self.connection.execute(
            "INSERT INTO application_settings(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=CURRENT_TIMESTAMP",
            params![PREFERENCE_KEY,encode(&next,4096)?]
        )?;
        transaction.commit()?;
        Ok(next)
    }

    // Call under a native read/write transaction. All facts and fingerprint
    // inputs come from one SQLite snapshot; no nested snapshot transaction.
    fn capture_strategy_inputs(&self) -> Result<AnalysisInputs> {
        let mut collection = Vec::new();
        let mut details = BTreeMap::new();
        let mut observations = Vec::new();
        let mut resources = BTreeMap::new();
        // Another application instance can activate a new package. Resolve the
        // saved identity pins with the package from this same database snapshot,
        // never old in-memory stats mixed with new combat/CP mechanics.
        let package = self.local_package().ok().flatten();
        let live_catalog = package.as_ref().and_then(|package| package.validate().ok());
        let package = package.filter(|_| live_catalog.is_some());
        let mut pokemons =
            self.list_with_catalog(live_catalog.as_ref().unwrap_or(&self.catalog))?;
        pokemons.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        for pokemon in pokemons {
            resources.insert(
                pokemon.id.as_str().to_owned(),
                self.preparation_resources(&pokemon.id)?,
            );
            let mut tags = self.tags_for(&pokemon.id)?;
            tags.sort_by_key(|tag| tag.id.as_i64());
            let detail = self.individual_details(&pokemon.id)?;
            observations.push(serde_json::json!({
                "id":pokemon.id.as_str(),"displayName":pokemon.display_name,
                "cp":pokemon.cp.map(|value|value.to_string()),"ivs":pokemon.ivs,
                "revision":pokemon.strategic_observation_revision.to_string(),
                "reference":pokemon.reference,"details":detail,
                "tags":tags.iter().map(|tag|serde_json::json!({"id":tag.id.as_i64().to_string(),"name":tag.name,"scope":format!("{:?}",tag.scope)})).collect::<Vec<_>>()
            }));
            details.insert(pokemon.id.as_str().to_owned(), detail);
            collection.push((pokemon, tags));
        }
        let mut lifecycle = BTreeMap::<String, String>::new();
        let mut statement = self.connection.prepare(
            "SELECT pokemon_id,request_id FROM pokemon_lifecycle_events WHERE rowid IN (SELECT max(rowid) FROM pokemon_lifecycle_events GROUP BY pokemon_id) ORDER BY pokemon_id"
        )?;
        for row in statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })? {
            let (id, request) = row?;
            lifecycle.insert(id, request);
        }
        let package_digest = package
            .as_ref()
            .map(|package| encode(package, 64 * 1024 * 1024).map(|text| digest(text.as_bytes())))
            .transpose()?;
        let mechanics = package.as_ref().map(|package| Mechanics {
            version: format!(
                "{}:{}",
                package.catalog["dataset"]["version"]
                    .as_str()
                    .unwrap_or("unknown"),
                package_digest.as_deref().unwrap_or("unknown")
            ),
            sha256: package_digest
                .clone()
                .unwrap_or_else(|| digest(b"unknown mechanics")),
            cp_multipliers: package
                .cp_multipliers
                .iter()
                .map(|&(level, multiplier)| CpMultiplier { level, multiplier })
                .collect(),
        });
        let preferences = self.strategy_preferences()?;
        let context = digest(
            &serde_json::to_vec(&serde_json::json!({
                "ruleset":strategy::RULESET_VERSION,"gymRuleset":crate::gym::RULESET_VERSION,"observations":observations,
                "latestLifecycle":lifecycle,"packageSha256":package_digest,
                "referenceError":self.reference_error,"preferences":preferences,"preparationResources":resources,
                "preparationRuleset":crate::preparation::VERSION,"rolesRuleset":crate::roles::RULESET_VERSION,
                "decisionRuleset":crate::decision::VERSION,"jointPlanningRuleset":crate::integrated_planning::VERSION
            }))
            .map_err(|_| Error::InvalidInput("Unable to fingerprint strategy inputs"))?,
        );
        Ok(AnalysisInputs {
            context,
            collection,
            details,
            mechanics,
            preferences,
            package,
            resources,
        })
    }

    pub fn collection_strategy(&self) -> Result<CollectionStrategy> {
        let transaction = self.connection.unchecked_transaction()?;
        let result = calculate(self.capture_strategy_inputs()?, None)?;
        transaction.commit()?;
        Ok(result)
    }

    pub(crate) fn individual_strategy_snapshot(
        &self,
        id: &PokemonId,
    ) -> Result<CollectionStrategy> {
        let transaction = self.connection.unchecked_transaction()?;
        self.get(id)?;
        let result = calculate(self.capture_strategy_inputs()?, Some(id.as_str()))?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn select_preparation_path(
        &self,
        id: &PokemonId,
        expected_context: &str,
        path_id: &str,
    ) -> Result<crate::preparation_resources::ResourceBudget> {
        let transaction =
            Transaction::new_unchecked(&self.connection, TransactionBehavior::Immediate)?;
        let inputs = self.capture_strategy_inputs()?;
        if inputs.context != expected_context {
            return Err(Error::ObservationConflict);
        }
        let mut budget = inputs
            .resources
            .get(id.as_str())
            .cloned()
            .ok_or(Error::NotFound("Pokemon"))?;
        let report = calculate(inputs, Some(id.as_str()))?
            .reports
            .into_iter()
            .find(|r| r.pokemon_id == id.as_str())
            .ok_or(Error::NotFound("Pokemon"))?;
        let path = report
            .preparation
            .as_ref()
            .and_then(|plan| plan.paths.iter().find(|path| path.id == path_id))
            .ok_or(Error::InvalidInput(
                "Preparation path is not in the current assessment",
            ))?;
        budget.values.selected_path_id = Some(path_id.into());
        budget.values.route_intent = Some(crate::preparation_resources::RouteIntent {
            dataset_id: report.dataset_id.clone(),
            target_form_id: path.target_form_id.clone(),
            roles: if path.served_roles.is_empty() {
                vec![path.role.clone()]
            } else {
                path.served_roles.clone()
            },
            target_level: path.target_level,
        });
        let next = self.save_preparation_resources_in_snapshot(
            id,
            parse_revision(&budget.revision)?,
            budget.values,
        )?;
        transaction.commit()?;
        Ok(next)
    }

    pub fn clear_preparation_route(
        &self,
        id: &PokemonId,
        expected_context: &str,
    ) -> Result<crate::preparation_resources::ResourceBudget> {
        let transaction =
            Transaction::new_unchecked(&self.connection, TransactionBehavior::Immediate)?;
        let inputs = self.capture_strategy_inputs()?;
        if inputs.context != expected_context {
            return Err(Error::ObservationConflict);
        }
        let mut budget = inputs
            .resources
            .get(id.as_str())
            .cloned()
            .ok_or(Error::NotFound("Pokemon"))?;
        budget.values.selected_path_id = None;
        budget.values.route_intent = None;
        let next = self.save_preparation_resources_in_snapshot(
            id,
            parse_revision(&budget.revision)?,
            budget.values,
        )?;
        transaction.commit()?;
        Ok(next)
    }

    fn stored_roadmap(&self, id: &PokemonId) -> Result<Option<StoredRoadmap>> {
        let row: Option<(String,String,Option<String>,String,i64,String)> = self.connection.query_row(
            "SELECT context,report_json,previous_json,progress_json,revision,generated_at FROM pokemon_roadmaps WHERE pokemon_id=?1",[id.as_str()],
            |row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?))
        ).optional()?;
        row.map(
            |(context, report, previous, progress, revision, generated_at)| {
                let report: StrategyReport = decode(&report, MAX_REPORT_BYTES)?;
                if report.pokemon_id != id.as_str() || revision < 0 {
                    return Err(Error::InvalidInput("Roadmap identity or revision mismatch"));
                }
                let progress = decode(&progress, MAX_PROGRESS_BYTES)?;
                validate_progress(&report, &progress)?;
                let previous: Option<PreviousRoadmap> = previous
                    .map(|text| decode(&text, MAX_PREVIOUS_BYTES))
                    .transpose()?;
                if let Some(previous) = &previous {
                    if previous.report.pokemon_id != id.as_str() {
                        return Err(Error::InvalidInput("Previous roadmap identity mismatch"));
                    }
                    validate_progress(&previous.report, &previous.progress)?;
                }
                Ok(StoredRoadmap {
                    context,
                    report,
                    previous,
                    progress,
                    revision,
                    generated_at,
                })
            },
        )
        .transpose()
    }

    pub fn roadmap(&self, id: &PokemonId) -> Result<RoadmapView> {
        let transaction = self.connection.unchecked_transaction()?;
        self.get(id)?;
        let context = self.capture_strategy_inputs()?.context;
        let result = match self.stored_roadmap(id)? {
            Some(stored) => stored.view(context),
            None => RoadmapView {
                context: context.clone(),
                current_context: context,
                revision: "0".into(),
                report: None,
                previous: None,
                progress: BTreeMap::new(),
                stale: false,
                generated_at: None,
            },
        };
        transaction.commit()?;
        Ok(result)
    }

    pub fn generate_roadmap(&self, id: &PokemonId, expected_context: &str) -> Result<RoadmapView> {
        let transaction =
            Transaction::new_unchecked(&self.connection, TransactionBehavior::Immediate)?;
        self.get(id)?;
        let inputs = self.capture_strategy_inputs()?;
        if inputs.context != expected_context {
            return Err(Error::ObservationConflict);
        }
        let old = self.stored_roadmap(id)?;
        if old
            .as_ref()
            .is_some_and(|old| old.context == inputs.context)
        {
            let result = old.unwrap().view(inputs.context);
            transaction.commit()?;
            return Ok(result);
        }
        let context = inputs.context.clone();
        let report = calculate(inputs, Some(id.as_str()))?
            .reports
            .into_iter()
            .find(|report| report.pokemon_id == id.as_str())
            .ok_or(Error::NotFound("Pokemon"))?;
        let report_json = encode(&report, MAX_REPORT_BYTES)?;
        let revision = old
            .as_ref()
            .map_or(Some(1), |old| old.revision.checked_add(1))
            .ok_or(Error::InvalidInput("Roadmap revision overflow"))?;
        let previous = old.map(|old| PreviousRoadmap {
            context: old.context,
            report: old.report,
            progress: old.progress,
            generated_at: old.generated_at,
        });
        let previous_json = previous
            .as_ref()
            .map(|previous| encode(previous, MAX_PREVIOUS_BYTES))
            .transpose()?;
        self.connection.execute(
            "INSERT INTO pokemon_roadmaps(pokemon_id,context,report_json,previous_json,progress_json,revision) VALUES (?1,?2,?3,?4,'{}',?5) ON CONFLICT(pokemon_id) DO UPDATE SET context=excluded.context,report_json=excluded.report_json,previous_json=excluded.previous_json,progress_json='{}',revision=excluded.revision,generated_at=CURRENT_TIMESTAMP",
            params![id.as_str(),context,report_json,previous_json,revision]
        )?;
        let result = self
            .stored_roadmap(id)?
            .ok_or(Error::NotFound("Roadmap"))?
            .view(context);
        transaction.commit()?;
        Ok(result)
    }

    pub fn update_roadmap_progress(
        &self,
        id: &PokemonId,
        expected_context: &str,
        expected_revision: i64,
        step_id: &str,
        status: &str,
    ) -> Result<RoadmapView> {
        if !valid_status(status) || step_id.len() > 200 {
            return Err(Error::InvalidInput(
                "Unknown roadmap progress status or step",
            ));
        }
        let transaction =
            Transaction::new_unchecked(&self.connection, TransactionBehavior::Immediate)?;
        self.get(id)?;
        let current_context = self.capture_strategy_inputs()?.context;
        let mut stored = self.stored_roadmap(id)?.ok_or(Error::NotFound("Roadmap"))?;
        if current_context != expected_context
            || stored.context != current_context
            || stored.revision != expected_revision
        {
            return Err(Error::ObservationConflict);
        }
        let step = stored
            .report
            .steps
            .iter()
            .find(|step| step.id == step_id)
            .ok_or(Error::InvalidInput("Unknown roadmap step"))?;
        if status == "completed" {
            if !matches!(step.action.as_str(), "compare" | "wait") {
                return Err(Error::InvalidInput("Use explicit observation or lifecycle confirmation; reporting completion does not change Pokémon facts"));
            }
            if !step.executable
                || step.prerequisites.iter().any(|id| {
                    stored
                        .progress
                        .get(id)
                        .is_none_or(|status| status != "completed")
                })
            {
                return Err(Error::InvalidInput(
                    "Complete the required confirmations before this planning step",
                ));
            }
        }
        let unchanged = if status == "pending" {
            !stored.progress.contains_key(step_id)
        } else {
            stored
                .progress
                .get(step_id)
                .is_some_and(|old| old == status)
        };
        if !unchanged {
            if status == "pending" {
                stored.progress.remove(step_id);
            } else {
                stored.progress.insert(step_id.into(), status.into());
            }
            validate_progress(&stored.report, &stored.progress)?;
            stored.revision = stored
                .revision
                .checked_add(1)
                .ok_or(Error::InvalidInput("Roadmap revision overflow"))?;
            self.connection.execute(
                "UPDATE pokemon_roadmaps SET progress_json=?2,revision=?3 WHERE pokemon_id=?1",
                params![
                    id.as_str(),
                    encode(&stored.progress, MAX_PROGRESS_BYTES)?,
                    stored.revision
                ],
            )?;
        }
        let result = stored.view(current_context);
        transaction.commit()?;
        Ok(result)
    }
}

fn calculate(inputs: AnalysisInputs, target: Option<&str>) -> Result<CollectionStrategy> {
    let mut reports = strategy::assess(&inputs.collection, inputs.mechanics.as_ref())
        .map_err(Error::InvalidInput)?;
    let gym_model = inputs
        .package
        .as_ref()
        .map(crate::gym::GymModel::new)
        .transpose()
        .map_err(Error::InvalidInput)?
        .flatten();
    let individuals: BTreeMap<_, _> = inputs
        .collection
        .iter()
        .map(|(pokemon, _)| (pokemon.id.as_str(), pokemon))
        .collect();
    let role_model = crate::roles::RoleModel::new(inputs.package.as_ref());
    let decision_model = crate::decision::DecisionModel::new(inputs.package.as_ref());
    for report in &mut reports {
        let Some(detail) = inputs.details.get(&report.pokemon_id) else {
            continue;
        };
        let assessment = gym_model.as_ref().map_or_else(
            || crate::gym::unavailable("Install a complete local PoGoAPI combat reference to assess gym attack potential."),
            |model| model.assess(individuals[report.pokemon_id.as_str()],Some(detail)));
        if let Some(build) = assessment.builds.first() {
            report.recommendations.push(StrategyRecommendation {
                id:"gym-attack-potential".into(),kind:"ATTACKER".into(),priority:18,
                explanation:assessment.summary.clone(),
                facts:vec![format!("Potential {} coverage with {} + {}; compared with {} ordinary reference forms, including unowned options.",build.attack_type,build.fast_move,build.charged_move,build.compared_forms)],
                missing:assessment.missing.clone(),alternative_pokemon_id:None,executable:false,
            });
            report.steps.insert(0,StrategyStep {id:"gym-review-moves".into(),action:"review".into(),required:false,
                prerequisites:vec![],rationale:format!("Optional gym preparation review: compare the reference set {} + {} with this individual's current attacks. Proposed moves are not observations. Acquisition/cost and actual readiness are unverified; confirm only facts you observed.",build.fast_move,build.charged_move),cost:None,executable:true});
            report.steps.insert(1,StrategyStep {id:"gym-compare-reference".into(),action:"compare".into(),required:false,
                prerequisites:vec![],rationale:format!("Review {} attack coverage against {} types and the external benchmark, then decide whether to keep this gym preparation option. This planning decision does not record a battle, spending or changed moves.",build.attack_type,if build.strong_against.is_empty(){"no established favorable".into()}else{build.strong_against.join(", ")}),cost:None,executable:true});
            if inputs.preferences.goal == "gym_attack" {
                report
                    .recommendations
                    .iter_mut()
                    .find(|item| item.id == "gym-attack-potential")
                    .unwrap()
                    .priority = 12;
            }
        }
        report.gym = Some(assessment);
        let mut retention = Vec::new();
        if detail.values.protected {
            retention.push("Explicit individual protection".to_string());
        }
        if detail.values.favorite == Some(true) {
            retention.push(
                "User-confirmed favorite state (separate from individual protection)".to_string(),
            );
        }
        if detail.values.shiny == Some(true) {
            retention.push("User-confirmed Shiny; no global rarity rate is inferred".to_string());
        }
        if detail.values.shadow == Some(true) {
            retention.push("User-confirmed Shadow; variant value remains distinct from generic combat statistics".to_string());
        }
        if detail.values.purified == Some(true) {
            retention.push(
                "User-confirmed Purified; retain pending personal collection review".to_string(),
            );
        }
        if !retention.is_empty() {
            report.protected = true;
            report
                .recommendations
                .retain(|recommendation| recommendation.kind != "EVOLVE");
            report.recommendations.push(StrategyRecommendation {
                id:"keep-confirmed-properties".into(),kind:"KEEP".into(),priority:0,
                explanation:"Retain this individual because of your confirmed properties or retention settings. No tags or favorites are changed by this suggestion.".into(),
                facts:retention,missing:Vec::new(),alternative_pokemon_id:None,executable:true,
            });
        }
        if detail.needs_reconfirmation {
            let rationale="Optional observations predate a confirmed action or correction. Review current moves, level and HP before using them for an investment.";
            report.recommendations.push(StrategyRecommendation {
                id: "review-optional-observations".into(),
                kind: "REVIEW MISSING INFORMATION".into(),
                priority: 8,
                explanation: rationale.into(),
                facts: vec!["Optional-observation reconfirmation flag".into()],
                missing: vec!["Explicitly reconfirm relevant current optional observations".into()],
                alternative_pokemon_id: None,
                executable: true,
            });
            report.steps.insert(
                0,
                StrategyStep {
                    id: "review-optional-observations".into(),
                    action: "review".into(),
                    required: true,
                    prerequisites: Vec::new(),
                    rationale: rationale.into(),
                    cost: None,
                    executable: true,
                },
            );
            if let Some(compare) = report
                .steps
                .iter_mut()
                .find(|step| step.id == "compare-alternatives")
            {
                compare
                    .prerequisites
                    .push("review-optional-observations".into());
                compare.executable = false;
            }
        }
        report.limitations.push(format!("Player goal: {}. Stardust budget: {}. Planning allocations never authorize or record spending.",inputs.preferences.goal,inputs.preferences.stardust_budget.map_or_else(||"unknown".into(),|value|value.to_string())));
        if inputs.preferences.stardust_budget == Some(0) {
            if let Some(wait) = report
                .recommendations
                .iter_mut()
                .find(|recommendation| recommendation.id == "wait-investment")
            {
                wait.priority = 6;
                wait.explanation="Your confirmed Stardust budget is zero. Keep useful collection information and avoid Stardust investment; no spending is inferred.".into();
            }
        }
        if inputs.preferences.goal == "collection" {
            for recommendation in &mut report.recommendations {
                if recommendation.kind == "EVOLVE" {
                    recommendation.priority = 15;
                }
            }
        }
        let selected_league = match inputs.preferences.goal.as_str() {
            "great" => Some(0),
            "ultra" => Some(1),
            "master" => Some(2),
            _ => None,
        };
        if let Some(index) = selected_league {
            if let Some(league) = report.leagues.get(index) {
                if let Some(compare) = report
                    .steps
                    .iter_mut()
                    .find(|step| step.id == "compare-alternatives")
                {
                    compare.rationale=format!("For your {} goal, compare this individual's supported statistical potential with actual owned alternatives of this form. Moves, eligibility, personal value and battle readiness still require review.",league.league);
                }
                if let Some(alternative) = &league.alternative_pokemon_id {
                    report.recommendations.push(StrategyRecommendation {
                        id:"compare-goal-alternative".into(),kind:"WAIT / NO ACTION".into(),priority:20,
                        explanation:format!("For your {} goal, another owned individual of this form has higher theoretical stat product within the same modeled level ceiling. Compare moves and personal collection value before investing.",league.league),
                        facts:league.potential.as_ref().map(|potential|vec![format!("This individual's modeled stat product: {:.3}; level ceiling {}",potential.stat_product,potential.level_ceiling)]).unwrap_or_default(),
                        missing:vec!["Confirmed moves, current level evidence, investment costs and goal-specific battle performance".into()],
                        alternative_pokemon_id:Some(alternative.clone()),executable:true,
                    });
                }
            }
        }
        report
            .recommendations
            .sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| a.id.cmp(&b.id)));
    }
    crate::gym::personalize(&mut reports);
    for report in &mut reports {
        if target.is_some_and(|id| id != report.pokemon_id) {
            continue;
        }
        let pokemon = individuals[report.pokemon_id.as_str()];
        let detail = inputs.details.get(&report.pokemon_id);
        report.roles = role_model.assess(pokemon, detail, &report.leagues);
        if report.protected {
            if let Some(transfer) = report.roles.iter_mut().find(|role| role.role == "transfer") {
                transfer.state = "retain".into();
                transfer.summary =
                    "Retain: your tags or confirmed individual properties require preservation."
                        .into();
                transfer.reasons.push("SAVE / SPECIAL, protection or confirmed retention properties override statistical comparisons.".into());
            }
        }
        for role in &report.roles {
            let kind = match role.role.as_str() {
                "gym_defense" => Some("GYM DEF"),
                "rocket" => Some("ROCKET"),
                _ => None,
            };
            if let Some(kind) = kind {
                if role.state != "insufficient_data" {
                    report.recommendations.push(StrategyRecommendation {
                        id: format!("role-{}", role.role),
                        kind: kind.into(),
                        priority: 24,
                        explanation: role.summary.clone(),
                        facts: role.reasons.clone(),
                        missing: role.missing.clone(),
                        alternative_pokemon_id: None,
                        executable: false,
                    });
                }
            }
        }
        if let Some(detail) = detail {
            let mut decision = decision_model.assess(report, pokemon, Some(detail));
            if let Some(alternative) = report
                .gym
                .as_ref()
                .and_then(|g| g.owned_alternative_pokemon_id.as_ref())
            {
                let name = individuals
                    .get(alternative.as_str())
                    .map(|p| p.display_name.as_str())
                    .unwrap_or("another owned individual");
                for use_ in decision
                    .suggestions
                    .iter_mut()
                    .filter(|s| s.tag == "ATTACKER")
                {
                    use_.evidence.push(format!("Owned comparison: {name} has higher equal-level potential for the current form's modeled attack type. Compare actual readiness and investment cost; this does not reduce this individual's external potential or imply either should be transferred."));
                }
            }
            let mut plan = crate::integrated_planning::candidates(
                pokemon,
                detail,
                inputs.package.as_ref(),
                gym_model.as_ref(),
                inputs
                    .resources
                    .get(&report.pokemon_id)
                    .cloned()
                    .unwrap_or_default(),
                inputs.preferences.stardust_budget,
                report.protected,
                &role_model,
                &decision,
            );
            let mut caps = Vec::new();
            match inputs.preferences.goal.as_str() {
                "great" => caps.push(1500),
                "ultra" => caps.push(2500),
                _ => {}
            }
            if let Some((_, tags)) = inputs.collection.iter().find(|(p, _)| p.id == pokemon.id) {
                for tag in tags {
                    match tag.name.as_str() {
                        "PVP 1500" => caps.push(1500),
                        "PVP 2500" => caps.push(2500),
                        _ => {}
                    }
                }
            }
            crate::integrated_planning::finalize(
                &mut plan,
                &decision,
                pokemon,
                detail,
                inputs.package.as_ref(),
                &caps,
            );
            // The integrated policy is authoritative for proposed tags. Legacy
            // raw role signals remain evidence, not unconditional assignments.
            for recommendation in &mut report.recommendations {
                if matches!(
                    recommendation.kind.as_str(),
                    "ATTACKER" | "GYM DEF" | "ROCKET"
                ) {
                    recommendation.kind = "REVIEW MISSING INFORMATION".into();
                    recommendation.explanation.push_str(" This is reference-model evidence. The integrated suggestions below identify the supported current or future form and its remaining conditions.");
                }
            }
            for suggestion in &decision.suggestions {
                if matches!(
                    suggestion.state.as_str(),
                    "supported_current" | "preparation_potential"
                ) {
                    report.recommendations.push(StrategyRecommendation {
                        id: format!("use-{}", suggestion.id),
                        kind: suggestion.tag.clone(),
                        priority: if suggestion.state == "supported_current" {
                            12
                        } else {
                            20
                        },
                        explanation: suggestion.explanation.clone(),
                        facts: suggestion.evidence.clone(),
                        missing: suggestion.missing.clone(),
                        alternative_pokemon_id: None,
                        executable: false,
                    });
                }
            }
            report.decision = Some(decision);
            // Alternatives are surfaced together; none is automatically chosen
            // and no user tag or observed game state is changed.
            for (kind, action) in [("EVOLVE", "evolve"), ("POWER UP", "power_up")] {
                if let Some(path) = plan
                    .selected_path_id
                    .as_ref()
                    .or(plan.primary_path_id.as_ref())
                    .and_then(|id| plan.paths.iter().find(|p| p.id == *id))
                    .filter(|p| {
                        p.actions.iter().any(|a| a.kind == action)
                            && p.affordability != "over_budget"
                    })
                {
                    report.recommendations.push(StrategyRecommendation{id:format!("preparation-{action}"),kind:kind.into(),priority:if report.protected{50}else{22},explanation:format!("Preparation alternative for {} at {}: {} Review the complete path and other roles before choosing.",path.target_name,path.target_level.map_or_else(||"an unconfirmed level".into(),|l|format!("level {l}")),path.benefit),facts:vec![format!("{} actions; allocation state {}",path.actions.len(),path.affordability)],missing:path.blockers.clone(),alternative_pokemon_id:None,executable:path.executable});
                }
            }
            if !plan.paths.is_empty() {
                report.steps.insert(0,StrategyStep{id:"review-preparation-paths".into(),action:"review".into(),required:false,prerequisites:vec![],rationale:"The app proposes a route from all supported uses. Review its evolution requirements, costs, attack acquisition and CP/move conflicts. Your existing route choice remains an optional override; no global role choice is required.".into(),cost:None,executable:true});
            }
            if let Some(path) = plan
                .selected_path_id
                .as_ref()
                .or(plan.primary_path_id.as_ref())
                .and_then(|id| plan.paths.iter().find(|p| p.id == *id))
            {
                let mut previous = vec!["review-preparation-paths".into()];
                let conflicts = plan
                    .conflicts
                    .iter()
                    .filter(|c| c.path_id == path.id)
                    .map(|c| c.explanation.clone())
                    .collect::<Vec<_>>();
                if !conflicts.is_empty() {
                    report.steps.insert(
                        1,
                        StrategyStep {
                            id: "review-compatible-uses".into(),
                            action: "review".into(),
                            required: true,
                            prerequisites: previous.clone(),
                            rationale: conflicts.join(" "),
                            cost: None,
                            executable: true,
                        },
                    );
                    previous = vec!["review-compatible-uses".into()];
                }
                for (index, action) in path.actions.iter().enumerate() {
                    if action.kind == "use_now" {
                        continue;
                    }
                    let id = format!("selected-preparation-{index}");
                    report.steps.push(StrategyStep{id:id.clone(),action:action.kind.clone(),required:true,prerequisites:previous.clone(),rationale:format!("{} {} {} After the real action, confirm it on this same individual. The app will reassess the remaining plan; progress never spends resources or invents observations.",action.description,action.requirements.join("; "),action.acquisition.join("; ")),cost:Some(format!("Complete path total: {} Candy, {} Candy XL, {} Stardust. {}",path.cost.candy.map_or_else(||"unknown".into(),|v|v.to_string()),path.cost.candy_xl.map_or_else(||"unknown".into(),|v|v.to_string()),path.cost.stardust.map_or_else(||"unknown".into(),|v|v.to_string()),path.cost.items.join("; "))),executable:path.executable});
                    previous = vec![id];
                }
            }
            report.preparation = Some(plan);
        }
        let alternative = report
            .gym
            .as_ref()
            .and_then(|gym| gym.owned_alternative_pokemon_id.clone());
        if let Some(alternative) = alternative {
            if let Some(recommendation) = report
                .recommendations
                .iter_mut()
                .find(|item| item.id == "gym-attack-potential")
            {
                recommendation.alternative_pokemon_id = Some(alternative);
                recommendation.priority = 26;
                recommendation.explanation.push_str(" An owned alternative has a higher cycle estimate for this attack type. This lowers preparation priority, not reference-based potential; actual readiness still requires review.");
            }
        }
        report
            .recommendations
            .sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| a.id.cmp(&b.id)));
    }
    if let Some(id) = target {
        reports.retain(|r| r.pokemon_id == id);
    }
    Ok(CollectionStrategy {
        context: inputs.context,
        reports,
        preferences: inputs.preferences,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::individual_details::IndividualDetails;
    use crate::inventory::{AppraisalIvs, PokemonInput, PowerUpConfirmation};
    use crate::reference::Identity;
    use crate::reference_store::LocalPackage;

    fn repository() -> (InventoryRepository, std::path::PathBuf, PokemonId) {
        let path = std::env::temp_dir().join(format!(
            "tcm-strategy-service-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut repo = InventoryRepository::open(&path).unwrap();
        let package = LocalPackage {
            catalog: serde_json::from_slice(include_bytes!(
                "../tests/fixtures/reference-synthetic.json"
            ))
            .unwrap(),
            cp_multipliers: (0..79)
                .map(|i| (1.0 + f64::from(i) / 2.0, 0.1 + f64::from(i) * 0.0075))
                .collect(),
            coverage: vec![],
            combat: None,
            preparation: None,
            trainer_combat: None,
            limitations: vec![],
            source_sha256: digest(b"original fictional strategy source"),
            provider: "fictional".into(),
            artifacts: vec![],
        };
        repo.install_reference(package).unwrap();
        let pokemon = repo
            .create(&PokemonInput {
                display_name: "Fictional collection member".into(),
                cp: None,
                ivs: Some(AppraisalIvs {
                    attack: 0,
                    defense: 0,
                    hp: 0,
                }),
            })
            .unwrap();
        repo.correct_identity(
            &pokemon.id,
            None,
            Some(Identity {
                species: "Testling".into(),
                form: Some("Normal".into()),
            }),
        )
        .unwrap();
        (repo, path, pokemon.id)
    }
    #[test]
    fn restart_keeps_plan_and_progress_without_modifying_observations() {
        let (repo, path, id) = repository();
        let context = repo.collection_strategy().unwrap().context;
        let plan = repo.generate_roadmap(&id, &context).unwrap();
        let before = repo.get(&id).unwrap();
        let progressed = repo
            .update_roadmap_progress(
                &id,
                &context,
                parse_revision(&plan.revision).unwrap(),
                "wait-investment",
                "completed",
            )
            .unwrap();
        assert_eq!(progressed.progress["wait-investment"], "completed");
        assert_eq!(repo.get(&id).unwrap(), before);
        assert!(repo
            .update_roadmap_progress(
                &id,
                &context,
                parse_revision(&plan.revision).unwrap(),
                "wait-investment",
                "ignored"
            )
            .is_err());
        let regenerated = repo.generate_roadmap(&id, &context).unwrap();
        assert_eq!(regenerated.progress, progressed.progress);
        assert_eq!(regenerated.revision, progressed.revision);
        drop(repo);
        let reopened = InventoryRepository::open(&path).unwrap();
        let saved = reopened.roadmap(&id).unwrap();
        assert!(!saved.stale);
        assert_eq!(saved.progress, progressed.progress);
        assert_eq!(saved.report.unwrap().pokemon_id, id.as_str());
        drop(reopened);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn history_only_power_up_stales_plan_even_without_cp_revision_change() {
        let (repo, path, id) = repository();
        let context = repo.collection_strategy().unwrap().context;
        let plan = repo.generate_roadmap(&id, &context).unwrap();
        let revision = repo.get(&id).unwrap().strategic_observation_revision;
        repo.confirm_power_up(
            &id,
            PowerUpConfirmation {
                request_id: "00000000-0000-4000-8000-000000000021",
                expected_revision: revision,
                cp: None,
                performed_in_game_confirmed: true,
            },
        )
        .unwrap();
        assert_eq!(
            repo.get(&id).unwrap().strategic_observation_revision,
            revision
        );
        assert!(repo.roadmap(&id).unwrap().stale);
        assert!(repo
            .update_roadmap_progress(
                &id,
                &context,
                parse_revision(&plan.revision).unwrap(),
                "wait-investment",
                "completed"
            )
            .is_err());
        let current = repo.collection_strategy().unwrap().context;
        let next = repo.generate_roadmap(&id, &current).unwrap();
        assert!(!next.stale);
        assert!(next.previous.is_some());
        assert!(next.progress.is_empty());
        drop(repo);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn preferences_protection_details_and_reference_versions_invalidate_context() {
        let (mut repo, path, id) = repository();
        let first = repo.collection_strategy().unwrap();
        repo.generate_roadmap(&id, &first.context).unwrap();
        let preferences = repo.save_strategy_preferences(0, "great", Some(0)).unwrap();
        assert_eq!(preferences.revision, "1");
        assert!(repo.roadmap(&id).unwrap().stale);
        assert!(repo.save_strategy_preferences(0, "great", None).is_err());
        let revision = repo.get(&id).unwrap().strategic_observation_revision;
        repo.save_individual_details(
            &id,
            revision,
            &repo.individual_details(&id).unwrap().evidence_token,
            IndividualDetails {
                shiny: Some(true),
                favorite: Some(true),
                protected: true,
                ..Default::default()
            },
        )
        .unwrap();
        let second = repo.collection_strategy().unwrap();
        assert_ne!(first.context, second.context);
        assert!(second.reports[0].protected);
        assert!(second.reports[0]
            .recommendations
            .iter()
            .any(|recommendation| recommendation.id == "keep-confirmed-properties"));
        assert!(second.reports[0]
            .recommendations
            .iter()
            .all(|recommendation| recommendation.kind != "EVOLVE"));
        assert!(repo.tags_for(&id).unwrap().is_empty());
        let mut package = repo.local_package().unwrap().unwrap();
        package.catalog["dataset"]["version"] = serde_json::json!("fictional-next");
        repo.install_reference(package).unwrap();
        assert_ne!(repo.collection_strategy().unwrap().context, second.context);
        drop(repo);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn review_reports_require_confirmation_and_generation_rejects_old_context() {
        let (repo, path, id) = repository();
        repo.correct_ivs(&id, repo.get(&id).unwrap().ivs, None)
            .unwrap();
        let context = repo.collection_strategy().unwrap().context;
        let plan = repo.generate_roadmap(&id, &context).unwrap();
        let revision = parse_revision(&plan.revision).unwrap();
        assert!(repo
            .update_roadmap_progress(&id, &context, revision, "review-ivs", "completed")
            .is_err());
        let requested = repo
            .update_roadmap_progress(&id, &context, revision, "review-ivs", "review_requested")
            .unwrap();
        assert_eq!(requested.progress["review-ivs"], "review_requested");
        assert!(repo.get(&id).unwrap().ivs.is_none());
        repo.create(&PokemonInput {
            display_name: "Another actual owned individual".into(),
            cp: None,
            ivs: None,
        })
        .unwrap();
        assert!(repo.generate_roadmap(&id, &context).is_err());
        assert!(repo.roadmap(&id).unwrap().stale);
        drop(repo);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn regeneration_bounds_previous_plan_and_failed_progress_does_not_write() {
        let (repo, path, id) = repository();
        let initial = repo.collection_strategy().unwrap().context;
        repo.generate_roadmap(&id, &initial).unwrap();
        for (revision, goal) in [(0, "great"), (1, "ultra"), (2, "master")] {
            repo.save_strategy_preferences(revision, goal, None)
                .unwrap();
            let context = repo.collection_strategy().unwrap().context;
            let plan = repo.generate_roadmap(&id, &context).unwrap();
            assert!(plan.previous.is_some());
            assert!(repo
                .update_roadmap_progress(
                    &id,
                    &context,
                    parse_revision(&plan.revision).unwrap(),
                    "unknown-step",
                    "ignored"
                )
                .is_err());
            assert!(repo.roadmap(&id).unwrap().progress.is_empty());
        }
        let previous: String = repo
            .connection
            .query_row(
                "SELECT previous_json FROM pokemon_roadmaps WHERE pokemon_id=?1",
                [id.as_str()],
                |row| row.get(0),
            )
            .unwrap();
        let json: serde_json::Value = serde_json::from_str(&previous).unwrap();
        assert!(json.get("previous").is_none());
        assert_eq!(
            repo.connection
                .query_row("SELECT count(*) FROM pokemon_roadmaps", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
        drop(repo);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn selected_goal_compares_actual_owned_alternative_without_assigning_tags() {
        let (repo, path, id) = repository();
        let other = repo
            .create(&PokemonInput {
                display_name: "Another fictional individual".into(),
                cp: None,
                ivs: Some(AppraisalIvs {
                    attack: 15,
                    defense: 15,
                    hp: 15,
                }),
            })
            .unwrap();
        repo.correct_identity(
            &other.id,
            None,
            Some(Identity {
                species: "Testling".into(),
                form: Some("Normal".into()),
            }),
        )
        .unwrap();
        repo.save_strategy_preferences(0, "master", None).unwrap();
        let result = repo.collection_strategy().unwrap();
        let report = result
            .reports
            .iter()
            .find(|report| report.pokemon_id == id.as_str())
            .unwrap();
        let recommendation = report
            .recommendations
            .iter()
            .find(|recommendation| recommendation.id == "compare-goal-alternative")
            .unwrap();
        assert_eq!(
            recommendation.alternative_pokemon_id.as_deref(),
            Some(other.id.as_str())
        );
        assert!(recommendation.explanation.contains("Master League"));
        assert!(report
            .steps
            .iter()
            .find(|step| step.action == "compare")
            .unwrap()
            .rationale
            .contains("Master League"));
        assert!(repo.tags_for(&id).unwrap().is_empty());
        drop(repo);
        let _ = std::fs::remove_file(path);
    }
}
