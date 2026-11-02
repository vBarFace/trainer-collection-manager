//! Bounded deterministic collection advice. These are statistical comparisons,
//! never battle rankings, game actions, or inferred observations.
use crate::inventory::{AppraisalIvs, Pokemon, Tag};
use crate::reference::{valid_text, BaseStats, ResolutionState};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const RULESET_VERSION: &str = "collection-3";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CpMultiplier {
    pub level: f64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Mechanics {
    pub version: String,
    pub sha256: String,
    pub cp_multipliers: Vec<CpMultiplier>,
}

impl Mechanics {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !valid_text(&self.version, 200)
            || self.sha256.len() != 64
            || !self
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || self.cp_multipliers.len() > 109
        {
            return Err("Invalid mechanics provenance or multiplier table size");
        }
        let mut previous_level = 0.0;
        let mut previous_multiplier = 0.0;
        for entry in &self.cp_multipliers {
            if !entry.level.is_finite()
                || !entry.multiplier.is_finite()
                || entry.level < 1.0
                || entry.level > 55.0
                || entry.level * 2.0 != (entry.level * 2.0).floor()
                || entry.level <= previous_level
                || entry.multiplier <= previous_multiplier
                || entry.multiplier > 1.0
            {
                return Err("CP multipliers must be finite, increasing half-level values");
            }
            previous_level = entry.level;
            previous_multiplier = entry.multiplier;
        }
        Ok(())
    }

    fn complete_grid_through(&self, ceiling: u16) -> bool {
        (0..(ceiling * 2 - 1)).all(|index| {
            self.cp_multipliers
                .get(usize::from(index))
                .is_some_and(|entry| entry.level == 1.0 + f64::from(index) / 2.0)
        })
    }

    pub fn modeled_level_ceiling(&self) -> Option<f64> {
        if self.complete_grid_through(50) {
            Some(50.0)
        } else if self.complete_grid_through(40) {
            Some(40.0)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LeaguePotential {
    pub level: f64,
    pub level_ceiling: f64,
    pub cp: u32,
    pub attack: f64,
    pub defense: f64,
    pub hp: u32,
    pub stat_product: f64,
    pub iv_rank: u16,
    pub ranked_spreads: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LeagueAssessment {
    pub league: String,
    pub cp_limit: Option<u32>,
    pub state: String,
    pub readiness: crate::utility::Readiness,
    pub potential: Option<LeaguePotential>,
    pub alternative_pokemon_id: Option<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyRecommendation {
    pub id: String,
    pub kind: String,
    pub priority: u8,
    pub explanation: String,
    pub facts: Vec<String>,
    pub missing: Vec<String>,
    pub alternative_pokemon_id: Option<String>,
    pub executable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyStep {
    pub id: String,
    pub action: String,
    pub required: bool,
    pub prerequisites: Vec<String>,
    pub rationale: String,
    pub cost: Option<String>,
    pub executable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyReport {
    pub pokemon_id: String,
    pub display_name: String,
    pub ruleset_version: String,
    pub model: crate::utility::ModelMode,
    pub observation_revision: String,
    pub dataset_id: String,
    pub dataset_version: String,
    pub mechanics_version: Option<String>,
    pub mechanics_sha256: Option<String>,
    pub protected: bool,
    pub unique_form: Option<bool>,
    pub duplicate_count: usize,
    pub duplicate_ids: Vec<String>,
    pub leagues: Vec<LeagueAssessment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gym: Option<crate::gym::GymAssessment>,
    pub recommendations: Vec<StrategyRecommendation>,
    pub steps: Vec<StrategyStep>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone)]
struct Stats {
    level: f64,
    cp: u32,
    attack: f64,
    defense: f64,
    hp: u32,
    product: f64,
}

// Formulas checked against PvPoke's Pokemon.js calculateCP, effective stats and
// generateIVCombinations. HP and CP have a minimum of 10 in GO. This scope
// excludes exceptional species mechanics and battle/Shadow modifiers.
// https://github.com/pvpoke/pvpoke/blob/master/src/js/pokemon/Pokemon.js
fn stats(base: &BaseStats, ivs: AppraisalIvs, entry: &CpMultiplier) -> Stats {
    let raw_attack = f64::from(base.attack) + f64::from(ivs.attack);
    let raw_defense = f64::from(base.defense) + f64::from(ivs.defense);
    let raw_stamina = f64::from(base.stamina) + f64::from(ivs.hp);
    let attack = raw_attack * entry.multiplier;
    let defense = raw_defense * entry.multiplier;
    let stamina = raw_stamina * entry.multiplier;
    let cp = ((raw_attack * raw_defense.sqrt() * raw_stamina.sqrt() * entry.multiplier.powi(2))
        / 10.0)
        .floor()
        .max(10.0) as u32;
    let hp = stamina.floor().max(10.0) as u32;
    Stats {
        level: entry.level,
        cp,
        attack,
        defense,
        hp,
        product: attack * defense * f64::from(hp),
    }
}

fn best_stats(
    base: &BaseStats,
    ivs: AppraisalIvs,
    multipliers: &[CpMultiplier],
    cp_limit: u32,
    level_ceiling: f64,
) -> Option<Stats> {
    // The native loader verifies the strictly increasing CPM table. Binary
    // search avoids rescanning every level for every one of 4,096 IV spreads.
    let usable = multipliers.partition_point(|entry| entry.level <= level_ceiling);
    let levels = &multipliers[..usable];
    let upper = levels.partition_point(|entry| stats(base, ivs, entry).cp <= cp_limit);
    upper
        .checked_sub(1)
        .map(|index| stats(base, ivs, &levels[index]))
}

type SpreadCache = HashMap<(u16, u16, u16, u32), Vec<f64>>;
type FormKey = (String, String, String);

fn potential(
    base: &BaseStats,
    ivs: AppraisalIvs,
    mechanics: &Mechanics,
    cap: u32,
    cache: &mut SpreadCache,
) -> Option<LeaguePotential> {
    let level_ceiling = mechanics.modeled_level_ceiling()?;
    let own = best_stats(base, ivs, &mechanics.cp_multipliers, cap, level_ceiling)?;
    // Bound derived working memory across collections with many distinct forms.
    if cache.len() >= 128 {
        cache.clear();
    }
    let products = cache
        .entry((base.attack, base.defense, base.stamina, cap))
        .or_insert_with(|| {
            let mut products = Vec::with_capacity(4096);
            for attack in 0..=15 {
                for defense in 0..=15 {
                    for hp in 0..=15 {
                        if let Some(value) = best_stats(
                            base,
                            AppraisalIvs {
                                attack,
                                defense,
                                hp,
                            },
                            &mechanics.cp_multipliers,
                            cap,
                            level_ceiling,
                        ) {
                            products.push(value.product);
                        }
                    }
                }
            }
            products
        });
    Some(LeaguePotential {
        level: own.level,
        level_ceiling,
        cp: own.cp,
        attack: own.attack,
        defense: own.defense,
        hp: own.hp,
        stat_product: own.product,
        iv_rank: 1 + products.iter().filter(|&&p| p > own.product + 1e-8).count() as u16,
        ranked_spreads: products.len() as u16,
    })
}

fn form_key(pokemon: &Pokemon) -> Option<FormKey> {
    if pokemon.reference.state != ResolutionState::Resolved {
        return None;
    }
    pokemon.reference.form.as_ref().map(|form| {
        (
            pokemon.reference.dataset.dataset_id.clone(),
            pokemon.reference.dataset.version.clone(),
            form.id.clone(),
        )
    })
}

fn exceptional_mechanics(pokemon: &Pokemon) -> Option<&'static str> {
    if pokemon
        .reference
        .species
        .as_ref()
        .is_some_and(|species| species.dex_number == 292)
    {
        return Some(
            "This species has exceptional HP mechanics which this ruleset does not evaluate.",
        );
    }
    let temporary_form = pokemon.reference.form.as_ref().is_some_and(|form| {
        [&form.id, &form.name].into_iter().any(|value| {
            value
                .to_ascii_lowercase()
                .split(|c: char| !c.is_ascii_alphanumeric())
                .any(|token| token == "mega" || token == "primal")
        })
    });
    temporary_form.then_some("Mega and Primal temporary forms require separate eligibility and transformation mechanics; ordinary league potential is unsupported.")
}

fn review(id: &str, explanation: &str, missing: Vec<String>) -> StrategyRecommendation {
    StrategyRecommendation {
        id: id.into(),
        kind: "REVIEW MISSING INFORMATION".into(),
        priority: 10,
        explanation: explanation.into(),
        facts: Vec::new(),
        missing,
        alternative_pokemon_id: None,
        executable: true,
    }
}

/// Pure, deterministic analysis of a consistent confirmed collection snapshot.
/// No mutation, network, clock, random input, inferred observation, or tag write.
pub fn assess(
    collection: &[(Pokemon, Vec<Tag>)],
    mechanics: Option<&Mechanics>,
) -> Result<Vec<StrategyReport>, &'static str> {
    if collection.len() > 20_000 {
        return Err("Collection exceeds the bounded analysis limit of 20,000 individuals");
    }
    if let Some(mechanics) = mechanics {
        mechanics.validate()?;
    }
    let mut ids = HashSet::new();
    let mut form_groups: HashMap<FormKey, Vec<usize>> = HashMap::new();
    for (index, (pokemon, _)) in collection.iter().enumerate() {
        if !ids.insert(pokemon.id.as_str())
            || pokemon.strategic_observation_revision < 0
            || pokemon.cp.is_some_and(|cp| cp <= 0)
            || pokemon
                .ivs
                .is_some_and(|ivs| ivs.attack > 15 || ivs.defense > 15 || ivs.hp > 15)
        {
            return Err("Invalid or duplicated confirmed individual");
        }
        if let Some(base) = pokemon
            .reference
            .form
            .as_ref()
            .and_then(|form| form.base_stats.as_ref())
        {
            if [base.attack, base.defense, base.stamina]
                .into_iter()
                .any(|stat| stat == 0)
            {
                return Err("Invalid GO base stats");
            }
        }
        if let Some(key) = form_key(pokemon) {
            form_groups.entry(key).or_default().push(index);
        }
    }
    let usable_mechanics = mechanics.filter(|m| m.modeled_level_ceiling().is_some());
    let mut cache = SpreadCache::new();
    let mut reports = Vec::with_capacity(collection.len());
    for (pokemon, tags) in collection {
        let resolved = pokemon.reference.state == ResolutionState::Resolved;
        let protected = tags.iter().any(|tag| tag.name == "SAVE/SPECIAL");
        let group = form_key(pokemon).and_then(|key| form_groups.get(&key));
        let duplicate_count = group.map_or(0, |group| group.len().saturating_sub(1));
        let mut duplicate_ids: Vec<String> = group
            .into_iter()
            .flatten()
            .map(|index| &collection[*index].0)
            .filter(|other| other.id != pokemon.id)
            .map(|other| other.id.as_str().into())
            .collect();
        duplicate_ids.sort();
        duplicate_ids.truncate(32);
        let mut report = StrategyReport {
            pokemon_id: pokemon.id.as_str().into(),
            display_name: pokemon.display_name.clone(),
            ruleset_version: RULESET_VERSION.into(),
            model: crate::utility::ModelMode::DeterministicOnly {},
            observation_revision: pokemon.strategic_observation_revision.to_string(),
            dataset_id: pokemon.reference.dataset.dataset_id.clone(),
            dataset_version: pokemon.reference.dataset.version.clone(),
            mechanics_version: mechanics.map(|m| m.version.clone()),
            mechanics_sha256: mechanics.map(|m| m.sha256.clone()),
            protected,
            unique_form: resolved.then_some(duplicate_count == 0),
            duplicate_count,
            duplicate_ids,
            leagues: Vec::new(),
            gym: None,
            recommendations: Vec::new(),
            steps: Vec::new(),
            limitations: vec![
                "Stat-product rank compares hypothetical IV spreads of the same form; it is not an overall battle ranking or a recommendation to spend resources.".into(),
                "Current level, moves, variant, eligibility and resource costs are not established by CP or IVs. Investment actions remain blocked.".into(),
                "PvE DPS/TDO, raid targets, gym defense, Rocket matchups, event availability and global rarity are unsupported in this ruleset.".into(),
                "Collection uniqueness refers to the confirmed species/form only. Shiny, Shadow, Purified, costume and personal value remain unresolved unless separately confirmed.".into(),
            ],
        };
        if protected {
            report.recommendations.push(StrategyRecommendation {
                id: "keep-protected".into(),
                kind: "KEEP".into(),
                priority: 0,
                explanation: "Retain this individual because you applied SAVE/SPECIAL.".into(),
                facts: vec!["User-applied SAVE/SPECIAL tag".into()],
                missing: Vec::new(),
                alternative_pokemon_id: None,
                executable: true,
            });
        } else if report.unique_form == Some(true) {
            report.recommendations.push(StrategyRecommendation {
                id: "keep-unique-form".into(), kind: "KEEP".into(), priority: 5,
                explanation: "This is your only confirmed individual of this species/form. Retain it while reviewing your collection goals.".into(),
                facts: vec!["No other confirmed individual resolves to the same form in this dataset namespace".into()],
                missing: vec!["Individual variant and personal collection value".into()],
                alternative_pokemon_id: None, executable: true,
            });
        }
        if !resolved {
            report.recommendations.push(review(
                "review-identity",
                "Confirm the species and form against an activated reference snapshot before comparing roles or duplicates.",
                vec!["Resolved confirmed species/form identity".into()],
            ));
        }
        if pokemon.ivs.is_none() {
            report.recommendations.push(review(
                "review-ivs",
                "Confirm all three appraisal IVs to calculate statistical league potential.",
                vec!["Attack, defense and HP appraisal IVs".into()],
            ));
        }
        let base = pokemon
            .reference
            .form
            .as_ref()
            .and_then(|form| form.base_stats.as_ref());
        let exceptional = exceptional_mechanics(pokemon);
        if resolved && base.is_none() {
            report.recommendations.push(review(
                "review-base-stats",
                "The active reference does not contain verified GO base stats for this form.",
                vec!["Pokémon GO base Attack, Defense and Stamina".into()],
            ));
        }
        if usable_mechanics.is_none() {
            report.recommendations.push(review(
                "review-mechanics", "Install a versioned CP multiplier table covering every half level from 1 through 40 (or 50) for bounded league calculations.",
                vec!["Complete level 1–40 or 1–50 CP multiplier table with provenance".into()],
            ));
        }
        let unsupported_cp = match (base, pokemon.ivs, usable_mechanics, pokemon.cp) {
            (Some(base), Some(ivs), Some(mechanics), Some(cp))
                if resolved && exceptional.is_none() =>
            {
                !mechanics
                    .cp_multipliers
                    .iter()
                    .filter(|entry| entry.level <= mechanics.modeled_level_ceiling().unwrap_or(0.0))
                    .any(|entry| i64::from(stats(base, ivs, entry).cp) == cp)
            }
            _ => false,
        };
        if unsupported_cp {
            report.recommendations.push(review(
                "review-cp-conflict", "Confirmed CP is not explained within the modeled level ceiling by the base stats and IVs. Review the observation or obtain mechanics for higher levels, special forms or Best Buddy. No level is inferred.",
                vec!["Consistent confirmed CP/IV/species observations or applicable special mechanics".into()],
            ));
        }
        for (league, cap) in [
            ("Great League", Some(1500)),
            ("Ultra League", Some(2500)),
            ("Master League", None),
        ] {
            let computed_explanation = format!("Hypothetical maximum stat product at half levels 1–{} under the CP cap. Theoretical IV rank includes all 0–15 spreads within this modeled ceiling, without encounter/trade IV floors, release restrictions or special cup eligibility. This is not a level-50 ranking when the ceiling is 40. Best Buddy levels are excluded. Current readiness and battle performance are unknown.", usable_mechanics.and_then(Mechanics::modeled_level_ceiling).unwrap_or(0.0));
            let observed_over_cap =
                cap.is_some_and(|cap| pokemon.cp.is_some_and(|cp| cp > i64::from(cap)));
            let computed =
                if resolved && exceptional.is_none() && !observed_over_cap && !unsupported_cp {
                    match (base, pokemon.ivs, usable_mechanics) {
                        (Some(base), Some(ivs), Some(mechanics)) => {
                            potential(base, ivs, mechanics, cap.unwrap_or(u32::MAX), &mut cache)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
            let (state, explanation) = if observed_over_cap {
                ("ineligible_observed_cp", "Confirmed CP exceeds this league's limit. Power Up cannot reduce CP; no lower-level preparation target is proposed.")
            } else if let Some(reason) = exceptional {
                ("unsupported_mechanics", reason)
            } else if unsupported_cp {
                ("conflicting_observation", "Confirmed CP is not explained within the modeled level ceiling by the base stats, IVs and table. Higher-level or special mechanics may be missing. Review before using statistical preparation targets.")
            } else if computed.is_some() {
                ("statistical_potential", computed_explanation.as_str())
            } else {
                ("insufficient_data", "League statistics need resolved identity, GO base stats, complete confirmed IVs and a complete versioned CP multiplier table.")
            };
            report.leagues.push(LeagueAssessment {
                league: league.into(),
                cp_limit: cap,
                state: state.into(),
                readiness: crate::utility::Readiness::InsufficientData,
                potential: computed,
                alternative_pokemon_id: None,
                explanation: explanation.into(),
            });
        }
        if let Some(form) = pokemon.reference.form.as_ref().filter(|_| resolved) {
            for evolution in form.evolutions.iter().flatten() {
                let already_owned = collection.iter().any(|(other, _)| {
                    other.reference.state == ResolutionState::Resolved
                        && other.reference.dataset.dataset_id
                            == pokemon.reference.dataset.dataset_id
                        && other
                            .reference
                            .form
                            .as_ref()
                            .is_some_and(|f| f.id == evolution.target_form_id)
                });
                if !already_owned && !protected {
                    report.recommendations.push(StrategyRecommendation {
                        id: format!("review-evolution-{}", evolution.target_form_id),
                        kind: "EVOLVE".into(), priority: 30,
                        explanation: format!("Evolution target {} has no confirmed individual in this local collection. Review whether filling that collection entry is worth changing this individual.", evolution.target_form_id),
                        facts: vec![format!("Reference evolution relationship: {} → {}", form.id, evolution.target_form_id)],
                        missing: vec!["Confirmed player permission to evolve this individual".into(), "Verified evolution prerequisites, costs and available resources".into(), "Variant eligibility and personal retention value".into()],
                        alternative_pokemon_id: report.duplicate_ids.first().cloned(), executable: false,
                    });
                }
            }
        }
        report.recommendations.push(StrategyRecommendation {
            id: "wait-investment".into(), kind: "WAIT / NO ACTION".into(), priority: 90,
            explanation: "Keep the current observations. Compare supported statistics and confirm the missing battle, variant and resource evidence before investing or considering disposal.".into(),
            facts: vec![format!("Confirmed CP: {}", pokemon.cp.map_or_else(|| "unknown".into(), |cp| cp.to_string()))],
            missing: vec!["Confirmed moves and level evidence".into(), "Goal-specific battle mechanics and known investment costs".into()],
            alternative_pokemon_id: None, executable: true,
        });
        for recommendation in &report.recommendations {
            if recommendation.kind == "REVIEW MISSING INFORMATION" {
                report.steps.push(StrategyStep {
                    id: recommendation.id.clone(),
                    action: "review".into(),
                    required: true,
                    prerequisites: Vec::new(),
                    rationale: recommendation.explanation.clone(),
                    cost: None,
                    executable: true,
                });
            }
        }
        report.steps.push(StrategyStep {
            id: "compare-alternatives".into(), action: "compare".into(), required: false,
            prerequisites: report.steps.iter().map(|step| step.id.clone()).collect(),
            rationale: "Compare individuals of this confirmed form in each supported league before choosing an investment. Different roles may favor different individuals.".into(),
            cost: None, executable: report.steps.is_empty(),
        });
        report.steps.push(StrategyStep {
            id: "wait-investment".into(), action: "wait".into(), required: false,
            prerequisites: Vec::new(), rationale: "Wait while moves, level, eligibility or costs remain unknown. Unknown cost never means zero cost.".into(),
            cost: None, executable: true,
        });
        report
            .recommendations
            .sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| a.id.cmp(&b.id)));
        reports.push(report);
    }
    // Compare actual owned individuals only within the same resolved form and
    // statistical league context. A higher product never authorizes disposal.
    for group in form_groups.values() {
        for league_index in 0..3 {
            let candidate = group
                .iter()
                .filter_map(|&index| {
                    reports[index].leagues[league_index]
                        .potential
                        .as_ref()
                        .map(|p| (reports[index].pokemon_id.clone(), p.stat_product))
                })
                .max_by(|a, b| a.1.total_cmp(&b.1).then_with(|| b.0.cmp(&a.0)));
            if let Some((id, product)) = candidate {
                for &index in group {
                    if reports[index].leagues[league_index]
                        .potential
                        .as_ref()
                        .is_some_and(|p| product > p.stat_product + 1e-8)
                    {
                        reports[index].leagues[league_index].alternative_pokemon_id =
                            Some(id.clone());
                    }
                }
            }
        }
    }
    reports.sort_by(|a, b| a.pokemon_id.cmp(&b.pokemon_id));
    Ok(reports)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{InventoryRepository, PokemonInput};
    use crate::reference::{Catalog, Identity};

    fn mechanics() -> Mechanics {
        // Original fictional mechanics table, not copied game data.
        Mechanics {
            version: "fictional-1".into(),
            sha256: "a".repeat(64),
            cp_multipliers: (0..99)
                .map(|i| CpMultiplier {
                    level: 1.0 + f64::from(i) / 2.0,
                    multiplier: 0.1 + f64::from(i) * 0.0075,
                })
                .collect(),
        }
    }
    fn collection() -> Vec<(Pokemon, Vec<Tag>)> {
        let dir = std::env::temp_dir().join(format!(
            "tcm-strategy-{}-{}.sqlite3",
            std::process::id(),
            std::thread::current()
                .name()
                .unwrap_or("test")
                .replace(':', "-")
        ));
        let _ = std::fs::remove_file(&dir);
        let mut repo = InventoryRepository::open(&dir).unwrap();
        let fixture: serde_json::Value =
            serde_json::from_slice(include_bytes!("../tests/fixtures/reference-synthetic.json"))
                .unwrap();
        repo.activate_reference(&serde_json::to_vec(&fixture).unwrap())
            .unwrap();
        let mut records = Vec::new();
        for (name, ivs) in [
            (
                "First",
                AppraisalIvs {
                    attack: 0,
                    defense: 0,
                    hp: 0,
                },
            ),
            (
                "Second",
                AppraisalIvs {
                    attack: 15,
                    defense: 15,
                    hp: 15,
                },
            ),
        ] {
            let pokemon = repo
                .create(&PokemonInput {
                    display_name: name.into(),
                    cp: None,
                    ivs: Some(ivs),
                })
                .unwrap();
            records.push((pokemon, Vec::new()));
        }
        // Construct a validated original form through Catalog to avoid private
        // inventory identity mutation or real game data in fixtures.
        let catalog = Catalog::load(&serde_json::to_vec(&fixture).unwrap()).unwrap();
        let species = fixture["species"][0]["name"].as_str().unwrap();
        let form = fixture["forms"][0]["name"].as_str().unwrap();
        for (pokemon, _) in &mut records {
            pokemon.reference = catalog.resolve(
                Some(&Identity {
                    species: species.into(),
                    form: Some(form.into()),
                }),
                None,
            );
            pokemon.reference.form.as_mut().unwrap().base_stats = Some(BaseStats {
                attack: 100,
                defense: 100,
                stamina: 100,
            });
        }
        drop(repo);
        let _ = std::fs::remove_file(&dir);
        records
    }

    #[test]
    fn independently_worked_numeric_example_cp_hp_and_stat_product() {
        // 100*sqrt(100)*sqrt(100)*.5²/10 = 250 CP;
        // effective attack = defense = HP = 50, product = 125,000.
        let value = stats(
            &BaseStats {
                attack: 100,
                defense: 100,
                stamina: 100,
            },
            AppraisalIvs {
                attack: 0,
                defense: 0,
                hp: 0,
            },
            &CpMultiplier {
                level: 10.0,
                multiplier: 0.5,
            },
        );
        assert_eq!(value.cp, 250);
        assert_eq!(value.hp, 50);
        assert_eq!(value.attack, 50.0);
        assert_eq!(value.defense, 50.0);
        assert_eq!(value.product, 125_000.0);
        let minimum = stats(
            &BaseStats {
                attack: 1,
                defense: 1,
                stamina: 1,
            },
            AppraisalIvs {
                attack: 0,
                defense: 0,
                hp: 0,
            },
            &CpMultiplier {
                level: 1.0,
                multiplier: 0.1,
            },
        );
        assert_eq!((minimum.cp, minimum.hp), (10, 10));
    }

    #[test]
    fn level_search_respects_cp_cap_and_excludes_best_buddy() {
        let levels = vec![
            CpMultiplier {
                level: 1.0,
                multiplier: 0.5,
            },
            CpMultiplier {
                level: 50.0,
                multiplier: 0.75,
            },
            CpMultiplier {
                level: 51.0,
                multiplier: 0.9,
            },
        ];
        let base = BaseStats {
            attack: 100,
            defense: 100,
            stamina: 100,
        };
        let ivs = AppraisalIvs {
            attack: 0,
            defense: 0,
            hp: 0,
        };
        let value = best_stats(&base, ivs, &levels, 250, 50.0).unwrap();
        assert_eq!((value.level, value.cp), (1.0, 250));
        assert!(best_stats(&base, ivs, &levels, 249, 50.0).is_none());
        assert_eq!(
            best_stats(&base, ivs, &levels, u32::MAX, 50.0)
                .unwrap()
                .level,
            50.0
        );
    }

    #[test]
    fn schema_valid_maximum_base_stats_do_not_overflow_with_ivs() {
        let value = stats(
            &BaseStats {
                attack: u16::MAX,
                defense: u16::MAX,
                stamina: u16::MAX,
            },
            AppraisalIvs {
                attack: 15,
                defense: 15,
                hp: 15,
            },
            &CpMultiplier {
                level: 50.0,
                multiplier: 0.5,
            },
        );
        // 65,535 + 15 = 65,550 before scaling, not a u16 addition.
        assert_eq!(value.attack, 32_775.0);
        assert_eq!(value.defense, 32_775.0);
        assert_eq!(value.hp, 32_775);
        assert_eq!(value.cp, 107_420_062);
        assert!(value.product.is_finite());
    }

    #[test]
    fn complete_level_forty_scope_is_useful_without_claiming_level_fifty_rank() {
        let records = collection();
        let mut table = mechanics();
        // A source's partial 41–45 coverage must not silently extend the
        // explicitly supported level-40 scope.
        table.cp_multipliers.truncate(89);
        assert_eq!(table.modeled_level_ceiling(), Some(40.0));
        let reports = assess(&records, Some(&table)).unwrap();
        for report in reports {
            let potential = report.leagues[2].potential.as_ref().unwrap();
            assert_eq!(potential.level_ceiling, 40.0);
            assert_eq!(potential.level, 40.0);
            assert_eq!(potential.ranked_spreads, 4096);
            assert!(report.leagues[2].explanation.contains("1–40"));
            assert!(report.leagues[2]
                .explanation
                .contains("not a level-50 ranking"));
        }
        table.cp_multipliers.truncate(78);
        assert_eq!(table.modeled_level_ceiling(), None);
        assert!(assess(&records, Some(&table))
            .unwrap()
            .iter()
            .all(|report| report
                .leagues
                .iter()
                .all(|league| league.potential.is_none())));
    }

    #[test]
    fn compares_owned_same_form_and_never_suggests_unsafe_disposal() {
        let collection = collection();
        let reports = assess(&collection, Some(&mechanics())).unwrap();
        let first = reports
            .iter()
            .find(|r| r.pokemon_id == collection[0].0.id.as_str())
            .unwrap();
        assert_eq!(first.unique_form, Some(false));
        assert_eq!(
            first.duplicate_ids,
            vec![collection[1].0.id.as_str().to_owned()]
        );
        assert_eq!(
            first.leagues[2].alternative_pokemon_id,
            Some(collection[1].0.id.as_str().into())
        );
        for report in &reports {
            assert!(report
                .recommendations
                .iter()
                .all(|r| !r.kind.contains("TRANSFER") && !r.kind.contains("TRADE")));
            assert!(report
                .recommendations
                .iter()
                .filter(|r| r.kind == "EVOLVE")
                .all(|r| !r.executable));
            assert_eq!(
                report.leagues[0].potential.as_ref().unwrap().ranked_spreads,
                4096
            );
        }
    }

    #[test]
    fn retained_unique_and_missing_inputs_produce_safe_explicit_states() {
        let mut records = collection();
        records.truncate(1);
        records[0].1.push(Tag {
            id: crate::inventory::TagId::parse("9").unwrap(),
            name: "SAVE/SPECIAL".into(),
            scope: crate::inventory::TagScope::Standard,
            created_at: String::new(),
        });
        records[0].0.ivs = None;
        let reports = assess(&records, None).unwrap();
        assert!(reports[0].protected);
        assert_eq!(reports[0].unique_form, Some(true));
        assert_eq!(reports[0].recommendations[0].id, "keep-protected");
        assert!(reports[0]
            .leagues
            .iter()
            .all(|l| l.state == "insufficient_data" && l.potential.is_none()));
        assert!(reports[0]
            .recommendations
            .iter()
            .any(|r| r.id == "review-ivs"));
        assert!(reports[0]
            .steps
            .iter()
            .any(|s| s.action == "wait" && s.executable && s.cost.is_none()));
    }

    #[test]
    fn observed_over_cap_does_not_propose_impossible_power_down() {
        let mut records = collection();
        records[0].0.cp = Some(2501);
        let reports = assess(&records, Some(&mechanics())).unwrap();
        let first = reports
            .iter()
            .find(|r| r.pokemon_id == records[0].0.id.as_str())
            .unwrap();
        assert_eq!(first.leagues[0].state, "ineligible_observed_cp");
        assert!(first.leagues[0].potential.is_none());
        assert!(first.leagues[1].potential.is_none());
        assert_eq!(first.leagues[2].state, "conflicting_observation");
    }

    #[test]
    fn malformed_and_incomplete_mechanics_and_duplicate_individuals_fail_safely() {
        let records = collection();
        let mut table = mechanics();
        table.cp_multipliers[0].multiplier = f64::NAN;
        assert!(assess(&records, Some(&table)).is_err());
        table = mechanics();
        table.cp_multipliers.truncate(10);
        assert!(assess(&records, Some(&table))
            .unwrap()
            .iter()
            .all(|r| r.leagues.iter().all(|l| l.potential.is_none())));
        let mut duplicates = records.clone();
        duplicates.push(records[0].clone());
        assert!(assess(&duplicates, None).is_err());
    }

    #[test]
    fn ambiguous_identity_and_exceptional_mechanics_abstain() {
        let mut records = collection();
        records[0].0.reference.state = ResolutionState::Ambiguous;
        records[1].0.reference.species.as_mut().unwrap().dex_number = 292;
        let reports = assess(&records, Some(&mechanics())).unwrap();
        let first = reports
            .iter()
            .find(|r| r.pokemon_id == records[0].0.id.as_str())
            .unwrap();
        assert_eq!(first.unique_form, None);
        assert!(first
            .recommendations
            .iter()
            .any(|r| r.id == "review-identity"));
        let second = reports
            .iter()
            .find(|r| r.pokemon_id == records[1].0.id.as_str())
            .unwrap();
        assert!(second
            .leagues
            .iter()
            .all(|l| l.state == "unsupported_mechanics"));
    }

    #[test]
    fn temporary_mega_primal_forms_abstain_using_whole_tokens_only() {
        let mut records = collection();
        records.truncate(1);
        records[0].0.reference.form.as_mut().unwrap().id = "fictional-mega-x".into();
        assert!(assess(&records, Some(&mechanics())).unwrap()[0]
            .leagues
            .iter()
            .all(|league| league.state == "unsupported_mechanics"));
        records[0].0.reference.form.as_mut().unwrap().id = "fictional-normal".into();
        records[0].0.reference.form.as_mut().unwrap().name = "Primal".into();
        assert!(assess(&records, Some(&mechanics())).unwrap()[0]
            .leagues
            .iter()
            .all(|league| league.state == "unsupported_mechanics"));
        records[0].0.reference.form.as_mut().unwrap().name = "Megamon".into();
        assert!(assess(&records, Some(&mechanics())).unwrap()[0]
            .leagues
            .iter()
            .all(|league| league.potential.is_some()));
    }

    #[test]
    fn deterministic_serialization_retains_unknown_cost_and_versions() {
        let records = collection();
        let a = serde_json::to_value(assess(&records, Some(&mechanics())).unwrap()).unwrap();
        let b = serde_json::to_value(assess(&records, Some(&mechanics())).unwrap()).unwrap();
        assert_eq!(a, b);
        assert_eq!(a[0]["rulesetVersion"], RULESET_VERSION);
        assert_eq!(a[0]["mechanicsSha256"], "a".repeat(64));
        assert!(a[0]["steps"][0]["cost"].is_null());
    }
}
