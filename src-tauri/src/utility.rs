//! S1 structural contracts only: no planner, persistence, IPC or game mechanics.
use crate::inventory::PokemonId;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};

pub const CONTRACT_VERSION: &str = "utility-1";
pub const MAX_DOCUMENT_BYTES: usize = 128 * 1024;
pub const MAX_PATHS: usize = 4;
pub const MAX_STEPS: usize = 8;
pub const MAX_ITEMS: usize = 16;

// Serde's default unit enums also accept {"variant": null}. Require strings
// explicitly so native deserialization has the same shape as TypeScript.
macro_rules! string_enum {
    ($name:ident { $($variant:ident => $wire:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
        pub enum $name { $(#[serde(rename = $wire)] $variant),+ }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D)
                -> std::result::Result<Self, D::Error>
            {
                let value = String::deserialize(deserializer)?;
                match value.as_str() {
                    $($wire => Ok(Self::$variant),)+
                    _ => Err(serde::de::Error::unknown_variant(&value, &[$($wire),+])),
                }
            }
        }
    };
}

string_enum!(Role {
    Attacker => "ATTACKER", GymDefense => "GYM DEF", Rocket => "ROCKET",
    Pvp1500 => "PVP 1500", Pvp2500 => "PVP 2500",
});
string_enum!(Readiness {
    UsableNow => "usable_now", UsableWithLimitations => "usable_with_limitations",
    PreparationRequired => "preparation_required", InsufficientData => "insufficient_data",
});
string_enum!(Validity {
    Current => "current", Blocked => "blocked", Stale => "stale", Invalid => "invalid",
});
string_enum!(PathKind { Prepare => "prepare", UseNow => "use_now", Wait => "wait" });
string_enum!(Affordability {
    Unknown => "unknown", WithinBudget => "within_budget", OverBudget => "over_budget",
});
string_enum!(ConditionState { Satisfied => "satisfied", Unmet => "unmet", Unknown => "unknown" });
string_enum!(ActionKind { Evolve => "evolve", PowerUp => "power_up", ChangeMoves => "change_moves" });
string_enum!(ActionTag { Evolve => "EVOLVE", PowerUp => "POWER UP" });

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SummaryTag {
    Role { tag: Role },
    Action { tag: ActionTag },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModelMode {
    DeterministicOnly {},
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Goal {
    pub id: String,
    pub role: Role,
    pub context_id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoleAssessment {
    pub role: Role,
    pub context_id: String,
    pub readiness: Readiness,
    pub explanation: String,
}

/// Decimal strings preserve the full nonnegative SQLite i64 range over JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceAmount {
    pub resource: String,
    #[serde(deserialize_with = "Option::<String>::deserialize")]
    pub amount: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Constraints {
    #[serde(deserialize_with = "Option::<Vec<ResourceAmount>>::deserialize")]
    pub budget: Option<Vec<ResourceAmount>>,
    #[serde(deserialize_with = "Option::<bool>::deserialize")]
    pub evolution_allowed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DatasetVersion {
    pub dataset_id: String,
    pub version: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceContext {
    pub observation_revision: String,
    pub constraint_revision: String,
    pub ruleset_version: String,
    #[serde(deserialize_with = "Option::<Vec<DatasetVersion>>::deserialize")]
    pub datasets: Option<Vec<DatasetVersion>>,
    pub model: ModelMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Provenance {
    pub evaluated: EvidenceContext,
    pub current: EvidenceContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvolutionTarget {
    pub dataset_id: String,
    pub species_id: String,
    pub form_id: String,
}

/// Bounded finite measurements, not observed CP or an inferred game threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Target {
    pub minimum: f64,
    pub maximum: f64,
    pub unit: String,
    pub context_id: String,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StepAction {
    Evolve { target: EvolutionTarget },
    PowerUp { target: Target },
    ChangeMoves { moves: Vec<String> },
    UseRole { role: Role },
    CollectEvidence { fields: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Condition {
    pub description: String,
    pub state: ConditionState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Step {
    pub id: String,
    pub required: bool,
    pub action: StepAction,
    pub prerequisites: Vec<String>,
    #[serde(deserialize_with = "Option::<Vec<Condition>>::deserialize")]
    pub conditions: Option<Vec<Condition>>,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Path {
    pub id: String,
    pub pokemon_id: String,
    pub goal_id: String,
    pub kind: PathKind,
    pub steps: Vec<Step>,
    #[serde(deserialize_with = "Option::<Vec<ResourceAmount>>::deserialize")]
    pub estimated_costs: Option<Vec<ResourceAmount>>,
    pub affordability: Affordability,
    pub expected_benefit: String,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NextAction {
    pub path_id: String,
    pub step_id: String,
    pub kind: ActionKind,
}

/// A user report only; it supplies no verified observations or actual spending.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActionReport {
    pub path_id: String,
    pub step_id: String,
    pub observation_revision: String,
    pub constraint_revision: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Assessment {
    pub contract_version: String,
    pub pokemon_id: String,
    pub goal: Goal,
    pub current_usefulness: String,
    #[serde(deserialize_with = "Option::<Vec<RoleAssessment>>::deserialize")]
    pub current_roles: Option<Vec<RoleAssessment>>,
    #[serde(deserialize_with = "Option::<Vec<RoleAssessment>>::deserialize")]
    pub potential_roles: Option<Vec<RoleAssessment>>,
    pub unsupported_roles: Vec<Role>,
    pub validity: Validity,
    pub validity_reasons: Vec<String>,
    #[serde(deserialize_with = "Option::<NextAction>::deserialize")]
    pub suggested_action: Option<NextAction>,
    #[serde(deserialize_with = "Option::<SummaryTag>::deserialize")]
    pub suggested_summary_tag: Option<SummaryTag>,
    pub selected_path_id: String,
    pub paths: Vec<Path>,
    pub constraints: Constraints,
    pub required_inputs: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub assumptions: Vec<String>,
    pub limitations: Vec<String>,
    pub provenance: Provenance,
    pub action_reports: Vec<ActionReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ValidationError {}

pub type Result<T> = std::result::Result<T, ValidationError>;

fn require(ok: bool, path: &str, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(ValidationError {
            path: path.into(),
            message: message.into(),
        })
    }
}

fn text(value: &str, path: &str) -> Result<()> {
    require(
        crate::reference::valid_text(value, 500),
        path,
        "expected trimmed text of 1–500 characters without controls",
    )
}

fn token(value: &str, path: &str, version: bool) -> Result<()> {
    require(
        !value.is_empty()
            && value.len() <= 64
            && value.as_bytes()[0].is_ascii_alphanumeric()
            && value.bytes().all(|b| {
                b.is_ascii_digit()
                    || b.is_ascii_lowercase()
                    || (version && b.is_ascii_uppercase())
                    || b"_-".contains(&b)
                    || (version && b == b'.')
                    || (!version && b == b':')
            }),
        path,
        "expected a 1–64 character identifier/version token",
    )
}

fn decimal(value: &str, path: &str) -> Result<i64> {
    require(
        !value.is_empty()
            && value.len() <= 19
            && value.bytes().all(|b| b.is_ascii_digit())
            && (value == "0" || !value.starts_with('0')),
        path,
        "expected a canonical nonnegative decimal integer string",
    )?;
    value.parse().map_err(|_| ValidationError {
        path: path.into(),
        message: "integer exceeds 9223372036854775807".into(),
    })
}

fn bounded(len: usize, max: usize, path: &str) -> Result<()> {
    require(len <= max, path, &format!("at most {max} entries allowed"))
}

fn texts(values: &[String], path: &str) -> Result<()> {
    bounded(values.len(), MAX_ITEMS, path)?;
    for (i, value) in values.iter().enumerate() {
        text(value, &format!("{path}[{i}]"))?;
    }
    Ok(())
}

fn amounts(values: &Option<Vec<ResourceAmount>>, path: &str) -> Result<()> {
    if let Some(values) = values {
        bounded(values.len(), MAX_ITEMS, path)?;
        let mut seen = HashSet::new();
        for (i, value) in values.iter().enumerate() {
            let p = format!("{path}[{i}]");
            token(&value.resource, &format!("{p}.resource"), false)?;
            require(seen.insert(&value.resource), &p, "duplicate resource")?;
            if let Some(amount) = &value.amount {
                decimal(amount, &format!("{p}.amount"))?;
            }
        }
    }
    Ok(())
}

fn context(value: &EvidenceContext, path: &str) -> Result<()> {
    decimal(
        &value.observation_revision,
        &format!("{path}.observationRevision"),
    )?;
    decimal(
        &value.constraint_revision,
        &format!("{path}.constraintRevision"),
    )?;
    token(
        &value.ruleset_version,
        &format!("{path}.rulesetVersion"),
        true,
    )?;
    if let Some(datasets) = &value.datasets {
        bounded(datasets.len(), MAX_ITEMS, &format!("{path}.datasets"))?;
        let mut seen = HashSet::new();
        for (i, dataset) in datasets.iter().enumerate() {
            let p = format!("{path}.datasets[{i}]");
            token(&dataset.dataset_id, &format!("{p}.datasetId"), false)?;
            token(&dataset.version, &format!("{p}.version"), true)?;
            require(
                seen.insert(&dataset.dataset_id),
                &p,
                "duplicate dataset namespace",
            )?;
            require(
                dataset.sha256.len() == 64
                    && dataset
                        .sha256
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
                &format!("{p}.sha256"),
                "expected a lowercase SHA-256 digest",
            )?;
        }
    }
    Ok(())
}

fn role_assessments(
    values: &Option<Vec<RoleAssessment>>,
    assessment: &Assessment,
    path: &str,
    potential: bool,
) -> Result<()> {
    if let Some(values) = values {
        bounded(values.len(), MAX_ITEMS, path)?;
        let mut seen = HashSet::new();
        for (i, value) in values.iter().enumerate() {
            let p = format!("{path}[{i}]");
            require(seen.insert(value.role), &p, "duplicate role/context")?;
            require(
                value.context_id == assessment.goal.context_id,
                &p,
                "role context differs from goal",
            )?;
            require(
                !assessment.unsupported_roles.contains(&value.role),
                &p,
                "unsupported role cannot have an assessment",
            )?;
            let readiness_matches = match value.readiness {
                Readiness::InsufficientData => true,
                Readiness::PreparationRequired => potential,
                Readiness::UsableNow | Readiness::UsableWithLimitations => !potential,
            };
            require(
                readiness_matches,
                &p,
                "current and potential readiness are distinct",
            )?;
            text(&value.explanation, &format!("{p}.explanation"))?;
        }
    }
    Ok(())
}

fn action_kind(action: &StepAction) -> Option<ActionKind> {
    match action {
        StepAction::Evolve { .. } => Some(ActionKind::Evolve),
        StepAction::PowerUp { .. } => Some(ActionKind::PowerUp),
        StepAction::ChangeMoves { .. } => Some(ActionKind::ChangeMoves),
        _ => None,
    }
}

fn step(value: &Step, assessment: &Assessment, path: &str) -> Result<()> {
    token(&value.id, &format!("{path}.id"), false)?;
    text(&value.explanation, &format!("{path}.explanation"))?;
    if let Some(conditions) = &value.conditions {
        bounded(conditions.len(), MAX_ITEMS, &format!("{path}.conditions"))?;
        for (i, condition) in conditions.iter().enumerate() {
            text(
                &condition.description,
                &format!("{path}.conditions[{i}].description"),
            )?;
        }
    }
    match &value.action {
        StepAction::Evolve { target } => {
            token(
                &target.dataset_id,
                &format!("{path}.action.target.datasetId"),
                false,
            )?;
            token(
                &target.species_id,
                &format!("{path}.action.target.speciesId"),
                false,
            )?;
            token(
                &target.form_id,
                &format!("{path}.action.target.formId"),
                false,
            )?;
        }
        StepAction::PowerUp { target } => {
            require(
                target.minimum.is_finite()
                    && target.maximum.is_finite()
                    && target.minimum >= 0.0
                    && target.maximum >= target.minimum
                    && target.maximum <= 1_000_000.0,
                &format!("{path}.action.target"),
                "expected a finite ordered measurement range within 0–1000000",
            )?;
            token(&target.unit, &format!("{path}.action.target.unit"), false)?;
            require(
                target.context_id == assessment.goal.context_id,
                path,
                "target context differs from goal",
            )?;
            text(
                &target.rationale,
                &format!("{path}.action.target.rationale"),
            )?;
        }
        StepAction::ChangeMoves { moves } => {
            require(
                !moves.is_empty(),
                path,
                "move-change action needs explicit moves",
            )?;
            texts(moves, &format!("{path}.action.moves"))?;
            require(
                moves.iter().collect::<HashSet<_>>().len() == moves.len(),
                path,
                "duplicate move",
            )?;
        }
        StepAction::UseRole { role } => {
            require(
                *role == assessment.goal.role,
                path,
                "use role differs from selected goal",
            )?;
        }
        StepAction::CollectEvidence { fields } => {
            require(
                !fields.is_empty(),
                path,
                "evidence action needs explicit fields",
            )?;
            texts(fields, &format!("{path}.action.fields"))?;
        }
    }
    Ok(())
}

/// Check declared per-path cost/budget assertions, never derive game costs or
/// combine mutually exclusive alternatives into one investment.
fn affordability(value: &Path, constraints: &Constraints, path: &str) -> Result<()> {
    if value.affordability == Affordability::Unknown {
        return Ok(());
    }
    let costs = value
        .estimated_costs
        .as_ref()
        .ok_or_else(|| ValidationError {
            path: format!("{path}.affordability"),
            message: "unknown costs cannot establish affordability".into(),
        })?;
    let mut over_budget = false;
    for cost in costs {
        let amount = cost.amount.as_ref().ok_or_else(|| ValidationError {
            path: format!("{path}.affordability"),
            message: "unknown resource cost cannot establish affordability".into(),
        })?;
        let amount = decimal(amount, path)?;
        if amount == 0 {
            continue;
        }
        let budget = constraints
            .budget
            .as_ref()
            .and_then(|v| v.iter().find(|b| b.resource == cost.resource))
            .and_then(|b| b.amount.as_ref())
            .ok_or_else(|| ValidationError {
                path: format!("{path}.affordability"),
                message: "positive cost needs a known budget for that resource".into(),
            })?;
        over_budget |= amount > decimal(budget, path)?;
    }
    require(
        over_budget == (value.affordability == Affordability::OverBudget),
        &format!("{path}.affordability"),
        "declared affordability contradicts this path's costs and budget",
    )
}

impl Assessment {
    pub fn load(bytes: &[u8], expected_pokemon_id: &str) -> Result<Self> {
        require(
            bytes.len() <= MAX_DOCUMENT_BYTES,
            "$",
            "contract exceeds 128 KiB",
        )?;
        let value: Self = serde_json::from_slice(bytes).map_err(|error| ValidationError {
            path: "$".into(),
            message: format!("invalid utility JSON: {error}"),
        })?;
        value.validate(expected_pokemon_id)?;
        Ok(value)
    }

    /// Structural validation only. Passing does not prove real game accuracy,
    /// eligibility, calculated costs, usefulness or user-confirmed completion.
    pub fn validate(&self, expected_pokemon_id: &str) -> Result<()> {
        require(
            self.contract_version == CONTRACT_VERSION,
            "contractVersion",
            "unsupported utility contract version",
        )?;
        require(
            PokemonId::parse(expected_pokemon_id).is_ok(),
            "expectedPokemonId",
            "expected canonical lowercase UUID v4",
        )?;
        require(
            PokemonId::parse(&self.pokemon_id).is_ok(),
            "pokemonId",
            "expected canonical lowercase UUID v4",
        )?;
        require(
            self.pokemon_id == expected_pokemon_id,
            "pokemonId",
            "individual association mismatch",
        )?;
        token(&self.goal.id, "goal.id", false)?;
        token(&self.goal.context_id, "goal.contextId", false)?;
        text(&self.goal.description, "goal.description")?;
        text(&self.current_usefulness, "currentUsefulness")?;
        bounded(self.unsupported_roles.len(), MAX_ITEMS, "unsupportedRoles")?;
        require(
            self.unsupported_roles.iter().collect::<HashSet<_>>().len()
                == self.unsupported_roles.len(),
            "unsupportedRoles",
            "duplicate unsupported role",
        )?;
        role_assessments(&self.current_roles, self, "currentRoles", false)?;
        role_assessments(&self.potential_roles, self, "potentialRoles", true)?;
        texts(&self.validity_reasons, "validityReasons")?;
        texts(&self.required_inputs, "requiredInputs")?;
        texts(&self.missing_evidence, "missingEvidence")?;
        texts(&self.assumptions, "assumptions")?;
        texts(&self.limitations, "limitations")?;
        amounts(&self.constraints.budget, "constraints.budget")?;
        context(&self.provenance.evaluated, "provenance.evaluated")?;
        context(&self.provenance.current, "provenance.current")?;
        if matches!(self.validity, Validity::Current | Validity::Blocked) {
            require(
                self.provenance.evaluated == self.provenance.current,
                "provenance",
                "changed context must be stale or invalid",
            )?;
        }
        if self.validity == Validity::Stale {
            require(
                self.provenance.evaluated != self.provenance.current,
                "provenance",
                "stale contract must identify changed input/version context",
            )?;
        }
        if self.validity != Validity::Current {
            require(
                !self.validity_reasons.is_empty(),
                "validityReasons",
                "non-current assessment needs a reason",
            )?;
            require(
                self.suggested_action.is_none() && self.suggested_summary_tag.is_none(),
                "suggestedAction",
                "non-current assessment must withhold actionable suggestions and tags",
            )?;
        }

        require(
            !self.paths.is_empty(),
            "paths",
            "at least one selected path is required",
        )?;
        bounded(self.paths.len(), MAX_PATHS, "paths")?;
        token(&self.selected_path_id, "selectedPathId", false)?;
        let mut path_ids = HashSet::new();
        let mut step_ids = HashSet::new();
        for (i, value) in self.paths.iter().enumerate() {
            let p = format!("paths[{i}]");
            token(&value.id, &format!("{p}.id"), false)?;
            require(path_ids.insert(&value.id), &p, "duplicate alternative ID")?;
            require(
                value.pokemon_id == self.pokemon_id,
                &p,
                "path individual association mismatch",
            )?;
            require(
                value.goal_id == self.goal.id,
                &p,
                "path goal association mismatch",
            )?;
            text(&value.expected_benefit, &format!("{p}.expectedBenefit"))?;
            texts(&value.limitations, &format!("{p}.limitations"))?;
            require(!value.steps.is_empty(), &p, "path needs at least one step")?;
            bounded(value.steps.len(), MAX_STEPS, &format!("{p}.steps"))?;
            let mut earlier = HashSet::new();
            for (j, s) in value.steps.iter().enumerate() {
                let sp = format!("{p}.steps[{j}]");
                step(s, self, &sp)?;
                require(step_ids.insert(&s.id), &sp, "duplicate plan-local step ID")?;
                bounded(
                    s.prerequisites.len(),
                    MAX_STEPS,
                    &format!("{sp}.prerequisites"),
                )?;
                let mut seen = HashSet::new();
                for prerequisite in &s.prerequisites {
                    require(seen.insert(prerequisite), &sp, "duplicate prerequisite")?;
                    require(earlier.contains(prerequisite), &format!("{sp}.prerequisites"), "prerequisite must name an earlier step in this path (no missing, forward, self or cyclic links)")?;
                }
                earlier.insert(&s.id);
            }
            let last = value.steps.last().expect("nonempty path checked");
            match value.kind {
                PathKind::Prepare => {
                    require(
                        matches!(last.action, StepAction::UseRole { .. })
                            && last.required
                            && value.steps[..value.steps.len() - 1]
                                .iter()
                                .all(|s| action_kind(&s.action).is_some())
                            && value.steps.len() > 1,
                        &p,
                        "preparation needs investment steps followed by required role use",
                    )?;
                }
                PathKind::UseNow => {
                    require(
                        value.steps.len() == 1 && matches!(last.action, StepAction::UseRole { .. }),
                        &p,
                        "use-now path has only role use",
                    )?;
                    require(
                        value
                            .estimated_costs
                            .as_ref()
                            .is_some_and(|v| v.iter().all(|c| c.amount.as_deref() == Some("0"))),
                        &p,
                        "use now requires known zero investment costs",
                    )?;
                }
                PathKind::Wait => {
                    require(
                        value
                            .steps
                            .iter()
                            .all(|s| matches!(s.action, StepAction::CollectEvidence { .. })),
                        &p,
                        "wait path offers evidence guidance only",
                    )?;
                }
            }
            amounts(&value.estimated_costs, &format!("{p}.estimatedCosts"))?;
            affordability(value, &self.constraints, &p)?;
        }
        let selected = self
            .paths
            .iter()
            .find(|p| p.id == self.selected_path_id)
            .ok_or_else(|| ValidationError {
                path: "selectedPathId".into(),
                message: "selected alternative does not exist".into(),
            })?;
        bounded(self.action_reports.len(), MAX_ITEMS, "actionReports")?;
        let mut reports = HashSet::new();
        for (i, report) in self.action_reports.iter().enumerate() {
            let p = format!("actionReports[{i}]");
            require(
                reports.insert((&report.path_id, &report.step_id)),
                &p,
                "duplicate action report",
            )?;
            require(
                self.paths.iter().any(|path| {
                    path.id == report.path_id && path.steps.iter().any(|s| s.id == report.step_id)
                }),
                &p,
                "action report must reference an existing path and step",
            )?;
            decimal(
                &report.observation_revision,
                &format!("{p}.observationRevision"),
            )?;
            decimal(
                &report.constraint_revision,
                &format!("{p}.constraintRevision"),
            )?;
        }
        if self.validity == Validity::Current {
            self.validate_current(selected)?;
        }
        Ok(())
    }

    fn validate_current(&self, selected: &Path) -> Result<()> {
        require(
            !self.unsupported_roles.contains(&self.goal.role),
            "goal.role",
            "unsupported goal cannot have a current recommendation",
        )?;
        match selected.kind {
            PathKind::Wait => require(false, "validity", "evidence-wait path must be blocked"),
            PathKind::UseNow => {
                require(
                    self.missing_evidence.is_empty(),
                    "missingEvidence",
                    "missing required evidence blocks a current role summary",
                )?;
                require(
                    selected.steps[0]
                        .conditions
                        .as_ref()
                        .is_some_and(|v| v.iter().all(|c| c.state == ConditionState::Satisfied)),
                    "suggestedSummaryTag",
                    "role use has unmet or unknown conditions",
                )?;
                require(
                    self.suggested_action.is_none(),
                    "suggestedAction",
                    "use now has no preparation action",
                )?;
                let ready = self.current_roles.as_ref().is_some_and(|v| {
                    v.iter().any(|r| {
                        r.role == self.goal.role
                            && matches!(
                                r.readiness,
                                Readiness::UsableNow | Readiness::UsableWithLimitations
                            )
                    })
                });
                require(
                    ready,
                    "currentRoles",
                    "use now requires a supported current role",
                )?;
                require(
                    self.suggested_summary_tag
                        == Some(SummaryTag::Role {
                            tag: self.goal.role,
                        }),
                    "suggestedSummaryTag",
                    "use now summarizes the supported role",
                )
            }
            PathKind::Prepare => {
                require(
                    self.missing_evidence.is_empty(),
                    "missingEvidence",
                    "missing required evidence blocks investment",
                )?;
                require(
                    selected.affordability == Affordability::WithinBudget,
                    "paths.affordability",
                    "actionable investment needs known affordability",
                )?;
                let potential = self.potential_roles.as_ref().is_some_and(|v| {
                    v.iter().any(|r| {
                        r.role == self.goal.role && r.readiness == Readiness::PreparationRequired
                    })
                });
                require(
                    potential,
                    "potentialRoles",
                    "preparation requires a supported potential role",
                )?;
                if selected
                    .steps
                    .iter()
                    .any(|s| matches!(s.action, StepAction::Evolve { .. }))
                {
                    require(
                        self.constraints.evolution_allowed == Some(true),
                        "constraints.evolutionAllowed",
                        "unknown or denied evolution permission blocks this path",
                    )?;
                    for step in &selected.steps {
                        if let StepAction::Evolve { target } = &step.action {
                            require(
                                self.provenance.evaluated.datasets.as_ref().is_some_and(
                                    |datasets| {
                                        datasets.iter().any(|d| d.dataset_id == target.dataset_id)
                                    },
                                ),
                                "provenance.evaluated.datasets",
                                "actionable evolution target needs recorded dataset provenance",
                            )?;
                        }
                    }
                }
                let next = self
                    .suggested_action
                    .as_ref()
                    .ok_or_else(|| ValidationError {
                        path: "suggestedAction".into(),
                        message: "current preparation needs a next action".into(),
                    })?;
                let first = &selected.steps[0];
                require(next.path_id == selected.id && next.step_id == first.id && Some(next.kind) == action_kind(&first.action), "suggestedAction", "next action must match the first step of the selected path; reports do not advance observations")?;
                require(
                    first.prerequisites.is_empty()
                        && first.conditions.as_ref().is_some_and(|v| {
                            v.iter().all(|c| c.state == ConditionState::Satisfied)
                        }),
                    "suggestedAction",
                    "next action has unmet or unknown conditions",
                )?;
                let expected = match next.kind {
                    ActionKind::Evolve => Some(SummaryTag::Action {
                        tag: ActionTag::Evolve,
                    }),
                    ActionKind::PowerUp => Some(SummaryTag::Action {
                        tag: ActionTag::PowerUp,
                    }),
                    ActionKind::ChangeMoves => None,
                };
                require(
                    self.suggested_summary_tag == expected,
                    "suggestedSummaryTag",
                    "summary tag must match the action; move changes have no default tag",
                )
            }
        }
    }
}

#[cfg(test)]
mod tests;
