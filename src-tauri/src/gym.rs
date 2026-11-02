//! Reference-based gym attack potential, followed separately by collection priority.
//! A bounded, fresh-energy attack cycle is not a battle simulation or readiness.
use crate::combat_reference::{CombatMove, CombatReference, TYPES};
use crate::individual_details::DetailsView;
use crate::inventory::Pokemon;
use crate::reference::{Form, ResolutionState, Species};
use crate::reference_store::LocalPackage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const RULESET_VERSION: &str = "gym-attack-1";
// Standard PvE damage model: floor(power * Atk / (2 * Def) * STAB * types) + 1.
// Both sides have the same hypothetical CPM, which cancels in Atk/Def.
// This version fixes neutral defender base Defense=200, Defense IV=15 and STAB=1.2.
const DEFENSE: f64 = 215.0;
const STAB: f64 = 1.2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GymBuild {
    pub attack_type: String,
    pub fast_move: String,
    pub charged_move: String,
    pub move_state: String,
    pub neutral_cycle_dps: f64,
    pub reference_percentile: f64,
    #[serde(default)]
    pub reference_summary: String,
    pub compared_forms: usize,
    pub reference_best: Vec<String>,
    pub strong_against: Vec<String>,
    pub resisted_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GymAssessment {
    pub state: String,
    pub ruleset_version: String,
    pub summary: String,
    pub pokemon_types: Vec<String>,
    pub vulnerable_to: Vec<String>,
    pub builds: Vec<GymBuild>,
    pub missing: Vec<String>,
    pub limitations: Vec<String>,
    pub owned_alternative_pokemon_id: Option<String>,
}

fn type_names(form: &Form) -> Vec<String> {
    form.types.as_ref().map_or_else(Vec::new, |types| {
        types
            .iter()
            .map(|kind| format!("{kind:?}").to_ascii_lowercase())
            .collect()
    })
}

fn damage(power: f64, attack: f64, defense: f64, stab: f64, effectiveness: f64) -> f64 {
    (power * attack / (2.0 * defense) * stab * effectiveness).floor() + 1.0
}

fn reference_summary(percentile: f64, count: usize) -> String {
    if count<10 {"Small imported reference subset: inspect the model details before drawing broad conclusions."}
    else if percentile>=75.0 {"Matches or exceeds at least three quarters of the imported same-type options in this neutral model."}
    else if percentile<=25.0 {"At least three quarters of the imported same-type options have higher output in this neutral model."}
    else {"Middle range of the imported same-type options in this neutral model."}.into()
}

#[derive(Clone, Copy)]
struct Pair<'a> {
    fast: &'a CombatMove,
    charged: &'a CombatMove,
    score: f64,
}

fn cycle(
    attack: f64,
    types: &[String],
    fast: &CombatMove,
    charged: &CombatMove,
    target_types: &[&str],
    reference: &CombatReference,
) -> Option<f64> {
    if fast.energy_delta <= 0 || charged.energy_delta >= 0 {
        return None;
    }
    let fast_count = ((-f64::from(charged.energy_delta)) / f64::from(fast.energy_delta)).ceil();
    let per_hit = |movement: &CombatMove| {
        let stab = if types.contains(&movement.move_type) {
            STAB
        } else {
            1.0
        };
        let effectiveness = target_types
            .iter()
            .map(|target| reference.effectiveness[&movement.move_type][*target])
            .product();
        damage(movement.power, attack, DEFENSE, stab, effectiveness)
    };
    let seconds =
        (fast_count * f64::from(fast.duration_ms) + f64::from(charged.duration_ms)) / 1000.0;
    Some((fast_count * per_hit(fast) + per_hit(charged)) / seconds)
}

struct ReferenceForm {
    name: String,
    form: Form,
}

pub struct GymModel<'a> {
    namespace: String,
    version: String,
    reference: &'a CombatReference,
    forms: BTreeMap<String, ReferenceForm>,
    moves: BTreeMap<&'a str, &'a CombatMove>,
    benchmarks: BTreeMap<String, Vec<(String, f64)>>,
}

fn exceptional(form: &Form, dex: u16) -> bool {
    // Transform, copied pools, exceptional HP and temporary form mechanics are
    // outside this ordinary-form model. Never rank these as generic attackers.
    [132, 235, 292].contains(&dex)
        || form.name.to_ascii_lowercase().contains("mega")
        || form.name.to_ascii_lowercase().contains("primal")
}

impl<'a> GymModel<'a> {
    pub fn new(package: &'a LocalPackage) -> Result<Option<Self>, &'static str> {
        let Some(reference) = package.combat.as_ref() else {
            return Ok(None);
        };
        package
            .validate()
            .map_err(|_| "Invalid gym reference package")?;
        let species: BTreeMap<String, Species> = package.catalog["species"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| {
                serde_json::from_value::<Species>(value.clone()).map(|s| (s.id.clone(), s))
            })
            .collect::<Result<_, _>>()
            .map_err(|_| "Invalid gym species")?;
        let mut forms = BTreeMap::new();
        for value in package.catalog["forms"].as_array().unwrap() {
            let form: Form =
                serde_json::from_value(value.clone()).map_err(|_| "Invalid gym form")?;
            let species = &species[&form.species_id];
            if form.base_stats.is_some()
                && form.types.is_some()
                && !exceptional(&form, species.dex_number)
            {
                forms.insert(
                    form.id.clone(),
                    ReferenceForm {
                        name: species.name.clone(),
                        form,
                    },
                );
            }
        }
        let moves = reference
            .moves
            .iter()
            .map(|movement| (movement.id.as_str(), movement))
            .collect();
        let mut model = Self {
            namespace: package.catalog["dataset"]["datasetId"]
                .as_str()
                .unwrap()
                .into(),
            version: package.catalog["dataset"]["version"]
                .as_str()
                .unwrap()
                .into(),
            reference,
            forms,
            moves,
            benchmarks: BTreeMap::new(),
        };
        for (id, entry) in &model.forms {
            let attack = f64::from(entry.form.base_stats.as_ref().unwrap().attack) + 15.0;
            for (kind, pair) in model.best_pairs(id, attack) {
                model
                    .benchmarks
                    .entry(kind)
                    .or_default()
                    .push((entry.name.clone(), pair.score));
            }
        }
        for benchmark in model.benchmarks.values_mut() {
            benchmark.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        }
        Ok(Some(model))
    }

    fn best_pairs(&self, form_id: &str, attack: f64) -> BTreeMap<String, Pair<'a>> {
        let mut result: BTreeMap<String, Pair<'a>> = BTreeMap::new();
        let Some(entry) = self.forms.get(form_id) else {
            return result;
        };
        let Some(pool) = self
            .reference
            .pools
            .iter()
            .find(|pool| pool.form_id == form_id)
        else {
            return result;
        };
        let types = type_names(&entry.form);
        for fast in pool
            .fast_move_ids
            .iter()
            .filter_map(|id| self.moves.get(id.as_str()))
        {
            if fast.power <= 0.0 {
                continue;
            }
            // Supplied nominal Hidden Power typing does not establish this
            // individual's hidden type. Transform requires separate mechanics.
            if ["hidden power", "transform"].contains(&fast.name.to_ascii_lowercase().as_str()) {
                continue;
            }
            for charged in pool
                .charged_move_ids
                .iter()
                .filter_map(|id| self.moves.get(id.as_str()))
            {
                if charged.power <= 0.0 {
                    continue;
                }
                let Some(score) = cycle(attack, &types, fast, charged, &[], self.reference) else {
                    continue;
                };
                let pair = Pair {
                    fast,
                    charged,
                    score,
                };
                let old = result.get(&charged.move_type);
                if old.is_none_or(|old| {
                    score > old.score
                        || score == old.score
                            && (&fast.id, &charged.id) < (&old.fast.id, &old.charged.id)
                }) {
                    result.insert(charged.move_type.clone(), pair);
                }
            }
        }
        result
    }

    pub fn assess(&self, pokemon: &Pokemon, detail: Option<&DetailsView>) -> GymAssessment {
        let mut result = unavailable(
            "Confirm an ordinary species/form identity to assess gym attack potential.",
        );
        if pokemon.reference.state != ResolutionState::Resolved
            || pokemon.reference.dataset.dataset_id != self.namespace
            || pokemon.reference.dataset.version != self.version
        {
            result
                .missing
                .push("Identity resolved within the active combat-reference namespace".into());
            return result;
        }
        let Some(form) = pokemon.reference.form.as_ref() else {
            return result;
        };
        result.pokemon_types = type_names(form);
        let Some(entry) = self.forms.get(&form.id) else {
            result.state = "unsupported".into();
            result.summary =
                "This form's stats, typing or special mechanics are outside the gym model.".into();
            return result;
        };
        let form = &entry.form;
        result.pokemon_types = type_names(form);
        if detail.is_some_and(|detail| detail.values.shadow == Some(true)) {
            result.state = "unsupported".into();
            result.summary="Shadow damage and variant-specific move restrictions are not supported by this ordinary-form gym model.".into();
            return result;
        }
        result.vulnerable_to = TYPES
            .iter()
            .filter(|kind| {
                result
                    .pokemon_types
                    .iter()
                    .map(|target| self.reference.effectiveness[**kind][target])
                    .product::<f64>()
                    > 1.0 + 1e-9
            })
            .map(|kind| (*kind).into())
            .collect();
        let attack = f64::from(form.base_stats.as_ref().unwrap().attack)
            + pokemon.ivs.map_or(15.0, |ivs| f64::from(ivs.attack));
        let pairs = self.best_pairs(&form.id, attack);
        let current = detail.filter(|detail| !detail.needs_reconfirmation);
        let observed = current.map(|detail| &detail.values);
        for (kind, pair) in pairs {
            let Some(benchmark) = self.benchmarks.get(&kind) else {
                continue;
            };
            let mut strong_against = Vec::new();
            let mut resisted_by = Vec::new();
            for target in TYPES {
                let score = cycle(
                    attack,
                    &result.pokemon_types,
                    pair.fast,
                    pair.charged,
                    &[target],
                    self.reference,
                )
                .unwrap();
                if self.reference.effectiveness[&kind][target] > 1.0 && score > pair.score + 1e-9 {
                    strong_against.push(target.into());
                }
                if score < pair.score - 1e-9 {
                    resisted_by.push(target.into());
                }
            }
            let confirmed = observed.is_some_and(|observed| {
                observed
                    .fast_move
                    .as_ref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(&pair.fast.name))
                    && [&observed.charged_move, &observed.second_charged_move]
                        .iter()
                        .any(|name| {
                            name.as_ref()
                                .is_some_and(|name| name.eq_ignore_ascii_case(&pair.charged.name))
                        })
            });
            let percentile = 100.0
                * benchmark
                    .iter()
                    .filter(|(_, score)| *score <= pair.score + 1e-9)
                    .count() as f64
                / benchmark.len() as f64;
            result.builds.push(GymBuild {
                attack_type: kind,
                fast_move: pair.fast.name.clone(),
                charged_move: pair.charged.name.clone(),
                move_state: if confirmed {
                    "confirmed"
                } else {
                    "hypothetical"
                }
                .into(),
                neutral_cycle_dps: pair.score,
                reference_percentile: percentile,
                reference_summary: reference_summary(percentile, benchmark.len()),
                compared_forms: benchmark.len(),
                reference_best: benchmark
                    .iter()
                    .take(3)
                    .map(|(name, _)| name.clone())
                    .collect(),
                strong_against,
                resisted_by,
            });
        }
        result.builds.sort_by(|a, b| {
            b.neutral_cycle_dps
                .total_cmp(&a.neutral_cycle_dps)
                .then_with(|| a.attack_type.cmp(&b.attack_type))
        });
        result.builds.truncate(3);
        if result.builds.is_empty() {
            result.missing.push(
                "Current fast/charged move pool with supported energy mechanics for this form"
                    .into(),
            );
            return result;
        }
        result.state = "limited".into();
        result.summary=format!("Gym-attack potential as a {} attacker under the fixed reference model. Review type coverage and proposed moves before deciding how to use this individual.",result.builds.iter().map(|build|build.attack_type.as_str()).collect::<Vec<_>>().join(" / "));
        result.missing.clear();
        if let Some(pool) = self
            .reference
            .pools
            .iter()
            .find(|pool| pool.form_id == form.id)
        {
            if !pool.withheld_moves.is_empty() {
                result.missing.push(format!("The installed source cannot model {} pool option(s): {}. Coverage and benchmarks are limited to supported moves, not all possible attacks.",pool.withheld_moves.len(),pool.withheld_moves.iter().take(6).cloned().collect::<Vec<_>>().join(", ")));
            }
        }
        if pokemon.ivs.is_none() {
            result.missing.push("Attack IV is unknown: the reference calculation uses a hypothetical 15, not an observation.".into());
        }
        if result
            .builds
            .iter()
            .all(|build| build.move_state != "confirmed")
        {
            result.missing.push("Proposed move sets are not confirmed on this individual. Optional manual move observations can identify matching reference sets.".into());
        }
        if detail.is_some_and(|detail| detail.needs_reconfirmation) {
            result.missing.push("Optional observations predate a confirmed action; reconfirm relevant moves and variant evidence.".into());
        }
        result.missing.push("Current level, battle readiness, move acquisition and investment costs remain unverified.".into());
        result
    }
}

pub fn unavailable(summary: &str) -> GymAssessment {
    GymAssessment {state:"needs_data".into(),ruleset_version:RULESET_VERSION.into(),summary:summary.into(),pokemon_types:vec![],vulnerable_to:vec![],builds:vec![],missing:vec![],owned_alternative_pokemon_id:None,
        limitations:vec![
            "Potential at an equal hypothetical level on both sides: the CP multiplier cancels. Defender base Defense 200 + IV15; neutral damage benchmark. Known Attack IV is used, otherwise a hypothetical 15. No current level is inferred.".into(),
            "One energy cycle from zero: ceil(charged energy / fast energy) fast moves and one charged move. Supplied millisecond durations; no incoming attacks, extra energy from damage, dodge, weather, friendship, motivation, fainting or carryover-energy simulation. This is not actual DPS, TDO or a win-rate.".into(),
            "Ordinary current damaging move pools only. Zero-power/status moves, legacy/event acquisition, dynamic move typing, special forms, Shadow modifiers, release status and gym placement eligibility are not established. Benchmarks describe imported reference coverage, not all obtainable Pokémon.".into(),
            "Favorable groups use single-type targets and both attacks in the modeled cycle. Dual typing and opponent moves can change results. Typing vulnerabilities are not a gym-defense evaluation. Rocket and defense require separate models.".into(),
        ]}
}

/// Collection changes personalize priority without altering the model values.
pub fn personalize(reports: &mut [crate::strategy::StrategyReport]) {
    let candidates: Vec<_> = reports
        .iter()
        .filter_map(|report| {
            report
                .gym
                .as_ref()
                .map(|gym| (&report.pokemon_id, gym.builds.clone()))
        })
        .collect();
    let alternatives: BTreeMap<String, String> = reports
        .iter()
        .filter_map(|report| {
            let best = report.gym.as_ref()?.builds.first()?;
            candidates
                .iter()
                .filter(|(id, _)| **id != report.pokemon_id)
                .filter_map(|(id, builds)| {
                    builds
                        .iter()
                        .find(|build| {
                            build.attack_type == best.attack_type
                                && build.neutral_cycle_dps > best.neutral_cycle_dps + 1e-9
                        })
                        .map(|build| ((*id).clone(), build.neutral_cycle_dps))
                })
                .max_by(|a, b| a.1.total_cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
                .map(|(id, _)| (report.pokemon_id.clone(), id))
        })
        .collect();
    for report in reports {
        if let Some(gym) = report.gym.as_mut() {
            gym.owned_alternative_pokemon_id = alternatives.get(&report.pokemon_id).cloned();
        }
    }
}

#[cfg(test)]
pub(crate) fn fictional_package() -> LocalPackage {
    use serde_json::json;
    let mut chart: BTreeMap<_, BTreeMap<_, _>> = TYPES
        .iter()
        .map(|a| {
            (
                (*a).to_string(),
                TYPES.iter().map(|d| ((*d).to_string(), 1.0)).collect(),
            )
        })
        .collect();
    // Original invented chart edges, not a committed real type-effectiveness dataset.
    chart.get_mut("normal").unwrap().insert("fire".into(), 1.6);
    chart
        .get_mut("normal")
        .unwrap()
        .insert("water".into(), 0.625);
    chart
        .get_mut("electric")
        .unwrap()
        .insert("normal".into(), 1.6);
    chart
        .get_mut("electric")
        .unwrap()
        .insert("grass".into(), 0.625);
    let raw = json!({
        "pokemon_stats":[
            {"pokemon_id":61001,"form":"Normal","pokemon_name":"Sampleton","base_attack":100,"base_defense":100,"base_stamina":100},
            {"pokemon_id":61002,"form":"Normal","pokemon_name":"Sampleprime","base_attack":200,"base_defense":100,"base_stamina":100},
            {"pokemon_id":61003,"form":"Normal","pokemon_name":"Samplelittle","base_attack":50,"base_defense":100,"base_stamina":100}],
        "pokemon_types":[{"pokemon_id":61001,"form":"Normal","type":["Normal","Grass"]},{"pokemon_id":61002,"form":"Normal","type":["Normal"]},{"pokemon_id":61003,"form":"Normal","type":["Normal"]}],
        "fast_moves":[{"move_id":900001,"name":"Sample Tap","type":"Normal","power":10,"energy_delta":10,"duration":1000}],
        "charged_moves":[{"move_id":900002,"name":"Sample Slam","type":"Normal","power":100,"energy_delta":-50,"duration":2000}],
        "current_pokemon_moves":[{"pokemon_id":61001,"form":"Normal","fast_moves":["Sample Tap"],"charged_moves":["Sample Slam"]},{"pokemon_id":61002,"form":"Normal","fast_moves":["Sample Tap"],"charged_moves":["Sample Slam"]},{"pokemon_id":61003,"form":"Normal","fast_moves":["Sample Tap"],"charged_moves":["Sample Slam"]}],
        "type_effectiveness":chart});
    let bytes = serde_json::to_vec(&raw).unwrap();
    let normalized =
        crate::source_adapter::normalize("pogoapi", &bytes, "original-gym-fixture-1", "2026-11-02")
            .unwrap();
    LocalPackage {
        catalog: normalized.catalog,
        cp_multipliers: normalized.cp_multipliers,
        combat: normalized.combat,
        coverage: normalized.coverage,
        limitations: normalized.limitations,
        source_sha256: crate::reference_store::digest(&bytes),
        provider: "pogoapi".into(),
        artifacts: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{AppraisalIvs, InventoryRepository, PokemonInput};
    use crate::reference::Identity;
    fn pokemon(name: &str) -> Pokemon {
        let path = std::env::temp_dir().join(format!(
            "gym-unit-{}-{}-{}.sqlite3",
            std::process::id(),
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut repo = InventoryRepository::open(&path).unwrap();
        let package = fictional_package();
        repo.install_reference(package).unwrap();
        let pokemon = repo
            .create(&PokemonInput {
                display_name: name.into(),
                cp: None,
                ivs: Some(AppraisalIvs {
                    attack: 15,
                    defense: 15,
                    hp: 15,
                }),
            })
            .unwrap();
        let result = repo
            .correct_identity(
                &pokemon.id,
                None,
                Some(Identity {
                    species: name.into(),
                    form: Some("Normal".into()),
                }),
            )
            .unwrap();
        drop(repo);
        let _ = std::fs::remove_file(path);
        result.0
    }
    #[test]
    fn independent_per_hit_and_fresh_energy_cycle_examples() {
        // P100, equal Atk/Def, STAB1.2: floor(60)+1=61.
        // With scalar1.6: floor(96)+1=97. Dual weakness: floor(153.6)+1=154.
        assert_eq!(damage(100.0, 100.0, 100.0, 1.2, 1.0), 61.0);
        assert_eq!(damage(100.0, 100.0, 100.0, 1.2, 1.6), 97.0);
        assert_eq!(damage(100.0, 100.0, 100.0, 1.2, 2.56), 154.0);
        let package = fictional_package();
        let reference = package.combat.as_ref().unwrap();
        // A215/D215: fast floor(6)+1=7, charged floor(60)+1=61.
        // Five fast attacks + one charge =96 damage in7sec.
        let score = cycle(
            215.0,
            &["normal".into()],
            &reference.moves[0],
            &reference.moves[1],
            &[],
            reference,
        )
        .unwrap();
        assert!((score - 96.0 / 7.0).abs() < 1e-12);
    }
    #[test]
    fn external_benchmark_and_type_guidance_work_without_owned_alternatives() {
        let package = fictional_package();
        let model = GymModel::new(&package).unwrap().unwrap();
        let report = model.assess(&pokemon("Sampleton"), None);
        assert_eq!(report.state, "limited");
        assert_eq!(report.builds[0].compared_forms, 3);
        assert!(report.builds[0]
            .reference_best
            .contains(&"Sampleprime".into()));
        assert!(report.builds[0].reference_percentile < 100.0);
        assert!(report.builds[0]
            .reference_summary
            .contains("Small imported reference subset"));
        assert_eq!(report.builds[0].strong_against, ["fire"]);
        assert_eq!(report.builds[0].resisted_by, ["water"]);
        // Invented dual typing cancels the two electric edges:1.6*.625=1.
        assert!(!report.vulnerable_to.contains(&"electric".into()));
        assert!(report.owned_alternative_pokemon_id.is_none());
        assert_eq!(report.builds[0].move_state, "hypothetical");
    }
    #[test]
    fn unknown_identity_variants_stale_observations_and_zero_energy_do_not_fabricate_readiness() {
        let package = fictional_package();
        let model = GymModel::new(&package).unwrap().unwrap();
        let mut individual = pokemon("Sampleton");
        individual.reference.state = ResolutionState::UnknownForm;
        assert!(model.assess(&individual, None).builds.is_empty());
        individual.reference.state = ResolutionState::Resolved;
        let mut detail = DetailsView {
            values: Default::default(),
            source: "manually_confirmed",
            observation_revision: "0".into(),
            saved_at_revision: None,
            needs_reconfirmation: false,
            evidence_token: "fictional".into(),
        };
        detail.values.fast_move = Some("Sample Tap".into());
        detail.values.charged_move = Some("Sample Slam".into());
        assert_eq!(
            model.assess(&individual, Some(&detail)).builds[0].move_state,
            "confirmed"
        );
        detail.needs_reconfirmation = true;
        assert_eq!(
            model.assess(&individual, Some(&detail)).builds[0].move_state,
            "hypothetical"
        );
        detail.values.shadow = Some(true);
        assert_eq!(
            model.assess(&individual, Some(&detail)).state,
            "unsupported"
        );
        let mut changed = package.clone();
        changed.combat.as_mut().unwrap().moves[0].energy_delta = 0;
        assert!(GymModel::new(&changed)
            .unwrap()
            .unwrap()
            .assess(&individual, None)
            .builds
            .is_empty());
        let mut changed = package.clone();
        changed.combat.as_mut().unwrap().moves[0].power = 0.0;
        assert!(GymModel::new(&changed)
            .unwrap()
            .unwrap()
            .assess(&individual, None)
            .builds
            .is_empty());
    }
}
