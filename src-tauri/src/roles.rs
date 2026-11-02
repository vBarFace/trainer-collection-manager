//! Independent role signals, never one overall quality score.
//! Gym motivation and placement context: https://niantic.helpshift.com/hc/en/6-pokemon-go/faq/83-battling-at-gyms/
//! Rocket uses Trainer Battles, not the gym damage cycle: https://niantic.helpshift.com/hc/en/6-pokemon-go/faq/1936-team-go-rocket-grunts/
use crate::combat_reference::{CombatReference, TYPES};
use crate::individual_details::DetailsView;
use crate::inventory::Pokemon;
use crate::reference::{Form, ResolutionState};
use crate::reference_store::LocalPackage;
use crate::strategy::LeagueAssessment;
use crate::trainer_combat_reference::{TrainerCombatReference, TrainerMove};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const RULESET_VERSION: &str = "roles-3";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoleAssessment {
    pub role: String,
    pub state: String,
    pub summary: String,
    pub reasons: Vec<String>,
    pub missing: Vec<String>,
}

fn signal(role: &str, summary: &str) -> RoleAssessment {
    RoleAssessment {
        role: role.into(),
        state: "insufficient_data".into(),
        summary: summary.into(),
        reasons: vec![],
        missing: vec![],
    }
}

fn types(form: &Form) -> Vec<String> {
    form.types.as_ref().map_or_else(Vec::new, |types| {
        types
            .iter()
            .map(|t| format!("{t:?}").to_ascii_lowercase())
            .collect()
    })
}

fn incoming_types(reference: &CombatReference, own: &[String], weak: bool) -> Vec<String> {
    if own.is_empty() {
        return vec![];
    }
    TYPES
        .iter()
        .filter(|attack| {
            let product = own
                .iter()
                .map(|defense| reference.effectiveness[**attack][defense])
                .product::<f64>();
            if weak {
                product > 1.0 + 1e-9
            } else {
                product < 1.0 - 1e-9
            }
        })
        .map(|kind| (*kind).into())
        .collect()
}

fn exceptional(form: &Form, dex: u16) -> bool {
    [132, 235, 292].contains(&dex)
        || form.name.to_ascii_lowercase().contains("mega")
        || form.name.to_ascii_lowercase().contains("primal")
}

#[derive(Clone, Copy)]
struct TrainerKit<'a> {
    fast: &'a TrainerMove,
    charged: &'a TrainerMove,
    first_fast_count: u32,
    first_fast_turns: u32,
}

fn supported_move(movement: &TrainerMove) -> bool {
    !["hidden power", "transform"].contains(&movement.name.to_ascii_lowercase().as_str())
}

/// One energy-oriented option per charged type. Earlier access to the first
/// charged move is the objective, not an asserted optimal battle or win rate.
fn trainer_kits<'a>(reference: &'a TrainerCombatReference, form_id: &str) -> Vec<TrainerKit<'a>> {
    let Some(pool) = reference.pools.iter().find(|pool| pool.form_id == form_id) else {
        return vec![];
    };
    let by_id: BTreeMap<_, _> = reference
        .moves
        .iter()
        .map(|movement| (movement.id.as_str(), movement))
        .collect();
    let mut by_type = BTreeMap::<&str, TrainerKit<'a>>::new();
    let prefer = |candidate: TrainerKit<'a>, previous: TrainerKit<'a>| {
        // Stable IDs break true metric ties, independently of provider record
        // order. These IDs do not imply a meaningful battle advantage.
        let charged_efficiency = |m: &TrainerMove| m.power / -f64::from(m.energy_delta);
        let fast_efficiency = |m: &TrainerMove| m.power / f64::from(m.turns);
        candidate
            .first_fast_turns
            .cmp(&previous.first_fast_turns)
            .then_with(|| {
                charged_efficiency(previous.charged)
                    .total_cmp(&charged_efficiency(candidate.charged))
            })
            .then_with(|| {
                fast_efficiency(previous.fast).total_cmp(&fast_efficiency(candidate.fast))
            })
            .then_with(|| {
                (&candidate.fast.id, &candidate.charged.id)
                    .cmp(&(&previous.fast.id, &previous.charged.id))
            })
            .is_lt()
    };
    for fast in pool
        .fast_move_ids
        .iter()
        .filter_map(|id| by_id.get(id.as_str()).copied())
        .filter(|movement| movement.energy_delta > 0 && supported_move(movement))
    {
        for charged in pool
            .charged_move_ids
            .iter()
            .filter_map(|id| by_id.get(id.as_str()).copied())
            .filter(|movement| {
                movement.energy_delta < 0 && movement.power > 0.0 && supported_move(movement)
            })
        {
            let cost = u32::from(charged.energy_delta.unsigned_abs());
            let gain = fast.energy_delta as u32;
            let first_fast_count = cost.div_ceil(gain);
            let candidate = TrainerKit {
                fast,
                charged,
                first_fast_count,
                first_fast_turns: first_fast_count * u32::from(fast.turns),
            };
            if by_type
                .get(charged.move_type.as_str())
                .is_none_or(|previous| prefer(candidate, *previous))
            {
                by_type.insert(charged.move_type.as_str(), candidate);
            }
        }
    }
    let mut result: Vec<_> = by_type.into_values().collect();
    result.sort_by(|a, b| {
        a.first_fast_turns
            .cmp(&b.first_fast_turns)
            .then_with(|| a.charged.move_type.cmp(&b.charged.move_type))
    });
    result.truncate(3);
    result
}

fn kit_description(kit: TrainerKit<'_>, detail: Option<&DetailsView>) -> String {
    let confirmed = detail
        .filter(|detail| !detail.needs_reconfirmation)
        .is_some_and(|detail| {
            detail
                .values
                .fast_move
                .as_ref()
                .is_some_and(|name| name.eq_ignore_ascii_case(&kit.fast.name))
                && [
                    &detail.values.charged_move,
                    &detail.values.second_charged_move,
                ]
                .iter()
                .any(|name| {
                    name.as_ref()
                        .is_some_and(|name| name.eq_ignore_ascii_case(&kit.charged.name))
                })
        });
    format!("{} energy-oriented {} option: {} + {}. Fast move: {:.2} energy/turn and {:.2} base power/turn; charged move: {:.2} base power/energy. From zero energy: {} fast moves, {} fast-action turns before the charged attack becomes affordable. This excludes charged-action time, shields, incoming damage, buffs, switches and Rocket interruptions.",
        if confirmed {"Confirmed-name"} else {"Hypothetical"},kit.charged.move_type,kit.fast.name,kit.charged.name,
        f64::from(kit.fast.energy_delta)/f64::from(kit.fast.turns),kit.fast.power/f64::from(kit.fast.turns),kit.charged.power/-f64::from(kit.charged.energy_delta),kit.first_fast_count,kit.first_fast_turns)
}

/// Construct once for an entire native collection snapshot. Source validation is
/// still enforced here so callers cannot accidentally mix unvalidated mechanics.
pub struct RoleModel<'a> {
    package: Option<&'a LocalPackage>,
    forms: BTreeMap<String, Form>,
    bulk: Vec<f64>,
}

impl<'a> RoleModel<'a> {
    pub fn new(package: Option<&'a LocalPackage>) -> Self {
        let package = package.filter(|package| package.validate().is_ok());
        let mut forms = BTreeMap::new();
        if let Some(package) = package {
            let dex: BTreeMap<String, u16> = package.catalog["species"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|row| {
                    Some((
                        row["id"].as_str()?.into(),
                        u16::try_from(row["dexNumber"].as_u64()?).ok()?,
                    ))
                })
                .collect();
            for row in package.catalog["forms"].as_array().unwrap() {
                if let Ok(form) = serde_json::from_value::<Form>(row.clone()) {
                    if form.base_stats.is_some() && !exceptional(&form, dex[&form.species_id]) {
                        forms.insert(form.id.clone(), form);
                    }
                }
            }
        }
        let bulk = forms
            .values()
            .map(|form| {
                let base = form.base_stats.as_ref().unwrap();
                (f64::from(base.defense) + 15.0) * (f64::from(base.stamina) + 15.0)
            })
            .collect();
        Self {
            package,
            forms,
            bulk,
        }
    }

    fn form(&self, pokemon: &Pokemon) -> Option<&Form> {
        let package = self.package?;
        if pokemon.reference.state != ResolutionState::Resolved
            || pokemon.reference.dataset.dataset_id
                != package.catalog["dataset"]["datasetId"].as_str()?
            || pokemon.reference.dataset.version
                != package.catalog["dataset"]["version"].as_str()?
        {
            return None;
        }
        self.forms.get(&pokemon.reference.form.as_ref()?.id)
    }

    pub fn assess(
        &self,
        pokemon: &Pokemon,
        detail: Option<&DetailsView>,
        leagues: &[LeagueAssessment],
    ) -> Vec<RoleAssessment> {
        let observed = detail
            .filter(|detail| !detail.needs_reconfirmation)
            .map(|detail| &detail.values);
        let form = self.form(pokemon);
        let reference = self.package.and_then(|package| package.combat.as_ref());
        let mut defense = signal("gym_defense", "Defense potential needs a confirmed identity and GO base stats from the active reference.");
        defense.missing = vec!["Gym placement eligibility and current gym context (free slot, existing species, motivation and care)".into(),
            "Opponent attacks and defender battle timing; no time-to-defeat or motivation-decay simulation is performed".into()];
        if let Some(form) = form {
            let base = form.base_stats.as_ref().unwrap();
            let bulk = (f64::from(base.defense)
                + pokemon.ivs.map_or(15.0, |ivs| f64::from(ivs.defense)))
                * (f64::from(base.stamina) + pokemon.ivs.map_or(15.0, |ivs| f64::from(ivs.hp)));
            let at_or_below = self.bulk.iter().filter(|value| **value <= bulk).count();
            defense.state = "conditional_potential".into();
            defense.summary = if self.bulk.len() < 10 {
                "Gym-defense potential uses a small imported reference subset. Review natural bulk and typing before choosing a defender."
            } else if at_or_below * 4 >= self.bulk.len() * 3 {
                "Stronger natural-bulk candidate in the imported equal-level stat model. Check typing, placement eligibility and gym context before choosing this defender."
            } else if at_or_below * 4 <= self.bulk.len() {
                "Lower natural bulk in the imported equal-level stat model. Compare other defenders unless typing or gym context favors this individual."
            } else {
                "Middle-range natural bulk among imported equal-level options. Type resistance, placement eligibility and gym context matter when choosing this defender."
            }.into();
            defense.reasons.push(format!("At equal hypothetical level, the Defense × Stamina proxy (with IVs) matches or exceeds {at_or_below} of {} imported ordinary-form options. This is an external stat comparison, not a defender battle ranking or placement-eligible shortlist.", self.bulk.len()));
            if self.bulk.len() < 10 {
                defense.reasons.push("The imported reference subset is small; this comparison does not establish broad defensive strength.".into());
            }
            if let Some(reference) = reference {
                let own_types = types(form);
                let weak = incoming_types(reference, &own_types, true);
                let resisted = incoming_types(reference, &own_types, false);
                if !resisted.is_empty() {
                    defense.reasons.push(format!(
                        "Typing resists incoming {} attacks; actual opponent move types matter.",
                        resisted.join(", ")
                    ));
                }
                if !weak.is_empty() {
                    defense.reasons.push(format!("Typing is vulnerable to incoming {} attacks; dual-type effects are multiplied.", weak.join(", ")));
                }
            } else {
                defense
                    .missing
                    .push("Imported complete type-effectiveness chart".into());
            }
            if pokemon.ivs.is_none() {
                defense.missing.push(
                    "Defense and HP IVs; comparison uses hypothetical 15/15, not observed values"
                        .into(),
                );
            }
            if observed.is_none_or(|values| values.level.is_none()) {
                defense.missing.push("Optional current level for current-state bulk; equal-level potential is not current readiness".into());
            }
            if let Some(values) = observed {
                match (values.current_hp, values.maximum_hp) {
                    (Some(current), Some(maximum)) if current < maximum => defense.reasons.push("Confirmed HP is below maximum: heal in the game before considering gym placement.".into()),
                    (Some(_), Some(_)) => defense.reasons.push("Current confirmed HP is full; gym placement still depends on game eligibility and gym context.".into()),
                    _ => defense.missing.push("Optional current and maximum HP for healing readiness".into()),
                }
                if values.shadow == Some(true) {
                    defense.missing.push("Shadow-specific incoming damage modifiers; ordinary bulk proxy does not model this variant".into());
                }
            }
        } else {
            defense.missing.push(
                "Confirmed active-reference identity and supported ordinary base stats".into(),
            );
        }

        let mut rocket = signal(
            "rocket",
            "Rocket potential needs confirmed identity, current move pools and type effectiveness.",
        );
        rocket.missing = vec!["Trainer-Battle move power, energy and turn timing; gym-cycle DPS is not a Rocket rating".into(),
            "Specific Grunt/Leader/Giovanni lineup, opponent attacks, shields and combat modifiers".into()];
        if let (Some(form), Some(reference)) = (form, reference) {
            let pool = reference.pools.iter().find(|pool| pool.form_id == form.id);
            if let Some(pool) = pool {
                let ids: BTreeSet<&str> = pool
                    .fast_move_ids
                    .iter()
                    .chain(&pool.charged_move_ids)
                    .map(String::as_str)
                    .collect();
                let attacks: BTreeSet<&str> = reference
                    .moves
                    .iter()
                    .filter(|movement| {
                        ids.contains(movement.id.as_str())
                            && movement.power > 0.0
                            && !["hidden power", "transform"]
                                .contains(&movement.name.to_ascii_lowercase().as_str())
                    })
                    .map(|movement| movement.move_type.as_str())
                    .collect();
                let favorable: Vec<&str> = TYPES
                    .iter()
                    .copied()
                    .filter(|target| {
                        attacks
                            .iter()
                            .any(|attack| reference.effectiveness[*attack][*target] > 1.0)
                    })
                    .collect();
                if !attacks.is_empty() {
                    rocket.state = "conditional_type_coverage".into();
                    rocket.summary = if favorable.is_empty() {
                        "Rocket option with imported damaging attacks; no favorable single-type coverage established.".into()
                    } else {
                        format!("Potential Rocket coverage against {} types; choose the actual opponent before preparing this option.", favorable.join(", "))
                    };
                    rocket.reasons.push("Coverage uses only move types from imported current pools. It does not imply these attacks are on this individual or that it wins a Rocket battle.".into());
                    let resisted = incoming_types(reference, &types(form), false);
                    if !resisted.is_empty() {
                        rocket.reasons.push(format!("Its typing resists incoming {} attacks. Resistance can help survivability when the opponent uses those move types.", resisted.join(", ")));
                    }
                    if !pool.withheld_moves.is_empty() {
                        rocket.missing.push(
                            "Some source move options were withheld; coverage is incomplete".into(),
                        );
                    }
                }
            }
            if observed
                .is_none_or(|values| values.fast_move.is_none() || values.charged_move.is_none())
            {
                rocket
                    .missing
                    .push("Optional confirmation of current Fast and Charged Attacks".into());
            }
            if observed.is_none_or(|values| values.level.is_none()) {
                rocket
                    .missing
                    .push("Optional current level for current readiness".into());
            }
        }

        let mut pvp = signal(
            "pvp",
            "PvP battle value is unresolved; league statistics are only one input.",
        );
        for league in leagues {
            if let Some(potential) = &league.potential {
                pvp.state = "statistical_potential_only".into();
                pvp.reasons.push(format!("{}: theoretical IV rank {}/{} within this form at modeled ceiling {} (target level {}, CP {}). This compares stat product, not species strength or win rate.", league.league, potential.iv_rank, potential.ranked_spreads, potential.level_ceiling, potential.level, potential.cp));
            } else if league.state == "ineligible_observed_cp" {
                pvp.reasons.push(format!("{}: confirmed CP is above its limit; evolution/Power Up cannot lower CP for this current form.", league.league));
            }
        }
        pvp.missing = vec!["Trainer-Battle move mechanics, current matchup/meta data and specific league/cup eligibility".into(),
            "Confirmed moves, level and applicable variant mechanics for actual battle readiness".into()];

        if let (Some(form), Some(trainer)) = (
            form,
            self.package
                .and_then(|package| package.trainer_combat.as_ref()),
        ) {
            let kits = trainer_kits(trainer, &form.id);
            if !kits.is_empty() {
                rocket.state = "trainer_move_potential".into();
                rocket.summary = format!("Rocket preparation options with {} charged attacks: compare energy access, coverage and resistance for the actual opponent.",kits.iter().map(|kit| kit.charged.move_type.as_str()).collect::<Vec<_>>().join(" / "));
                rocket.reasons.clear();
                rocket
                    .missing
                    .retain(|missing| !missing.starts_with("Trainer-Battle move power"));
                pvp.state = if leagues.iter().any(|league| league.potential.is_some()) {
                    "stat_and_move_potential"
                } else {
                    "move_kit_potential"
                }
                .into();
                pvp.summary = "PvP preparation potential combines move energy access and coverage with each league's supported stat-product targets.".into();
                pvp.missing[0] = "Current matchup/meta data, shields, baiting, opponent attacks and specific league/cup eligibility".into();
                for kit in &kits {
                    let description = kit_description(*kit, detail);
                    rocket.reasons.push(description.clone());
                    pvp.reasons.push(description);
                    if let Some(reference) = reference {
                        let strong: Vec<_> = TYPES
                            .iter()
                            .filter(|target| {
                                reference.effectiveness[&kit.charged.move_type][**target] > 1.0
                            })
                            .copied()
                            .collect();
                        let resisted: Vec<_> = TYPES
                            .iter()
                            .filter(|target| {
                                reference.effectiveness[&kit.charged.move_type][**target] < 1.0
                            })
                            .copied()
                            .collect();
                        let coverage = format!("{} charged-attack typing is favorable against {} and resisted by {} single types. Dual typing and the fast attack can change the full matchup.",kit.charged.move_type,if strong.is_empty(){"no established".into()}else{strong.join(", ")},if resisted.is_empty(){"no established".into()}else{resisted.join(", ")});
                        rocket.reasons.push(coverage.clone());
                        pvp.reasons.push(coverage);
                    }
                    if kit.fast.has_buffs || kit.charged.has_buffs {
                        rocket.missing.push(format!("{} + {} has supplied buff/debuff mechanics not simulated by this energy-access model",kit.fast.name,kit.charged.name));
                        pvp.missing.push("Selected move options can alter combat stats; buff/debuff effects are not simulated".into());
                    }
                }
                if let Some(reference) = reference {
                    let resisted = incoming_types(reference, &types(form), false);
                    if !resisted.is_empty() {
                        rocket.reasons.push(format!("Its typing resists incoming {} attacks; check the actual opponent's move types.",resisted.join(", ")));
                    }
                } else {
                    rocket
                        .missing
                        .push("Imported type-effectiveness chart for matchup coverage".into());
                    pvp.missing
                        .push("Imported type-effectiveness chart for matchup coverage".into());
                }
                rocket.reasons.push("These are bounded energy-oriented alternatives, not a simulated best Rocket team, guaranteed shield removal or predicted win.".into());
                pvp.reasons.push("Base move power/turn and power/energy are move-efficiency measures, not actual damage, overall species rankings or win rates. STAB, stats and effectiveness must be applied for a specific opponent.".into());
                if trainer
                    .pools
                    .iter()
                    .find(|pool| pool.form_id == form.id)
                    .is_some_and(|pool| !pool.withheld_moves.is_empty())
                {
                    rocket.missing.push("Some Trainer-Battle pool options are unavailable or ambiguous in the imported mechanics".into());
                    pvp.missing.push("Some Trainer-Battle pool options are unavailable or ambiguous in the imported mechanics".into());
                }
            }
        }

        let mut transfer = signal(
            "transfer",
            "Retain for review: the available evidence does not establish that transfer is safe.",
        );
        transfer.state = "retention_review".into();
        transfer.reasons.push("Weakness in one role, low CP or low IV percentage never establishes no value across the other roles.".into());
        // Retention protections survive stale battle observations. Staleness can
        // block readiness, but must never silently remove a disposal safeguard.
        if detail.is_some_and(|detail| {
            let values = &detail.values;
            values.protected
                || values.favorite == Some(true)
                || values.shiny == Some(true)
                || values.shadow == Some(true)
                || values.purified == Some(true)
        }) {
            transfer.state = "retain_protected_value".into();
            transfer.summary = "Retain: this individual has a confirmed protection, favorite or special-variant retention signal.".into();
        }
        if let Some(form) = form {
            if form
                .evolutions
                .as_ref()
                .is_some_and(|evolutions| !evolutions.is_empty())
            {
                transfer.reasons.push("The reference contains evolution branches: assess their future roles and preparation cost before disposal.".into());
            } else if form.evolutions.is_none() {
                transfer.missing.push("Verified evolution graph; missing relationships do not establish a final evolution".into());
            }
        }
        transfer.missing.extend([
            "Resolved future roles, acquisition/retention value and collection alternatives".into(),
            "Personal confirmation before any irreversible transfer in the game".into(),
        ]);
        if detail.is_some_and(|detail| detail.needs_reconfirmation) {
            for role in [&mut defense, &mut rocket, &mut pvp, &mut transfer] {
                role.missing.push("Optional observations predate a confirmed action; reconfirm the relevant individual facts".into());
            }
        }
        vec![defense, rocket, pvp, transfer]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{AppraisalIvs, PokemonId};
    use crate::reference::Identity;

    fn pokemon(package: &LocalPackage) -> Pokemon {
        Pokemon {
            id: PokemonId::parse("00000000-0000-4000-8000-000000000001").unwrap(),
            display_name: "Sampleton".into(),
            cp: None,
            strategic_observation_revision: 0,
            ivs: Some(AppraisalIvs {
                attack: 15,
                defense: 15,
                hp: 15,
            }),
            created_at: "test".into(),
            updated_at: "test".into(),
            evaluation: crate::evaluation::Evaluation {
                scope: crate::evaluation::EvaluationScope::Appraisal,
                state: crate::evaluation::EvaluationState::Pending,
                current_ruleset_version: crate::evaluation::RULESET_VERSION,
                ruleset_version: None,
                evaluated_at: None,
                result: None,
            },
            reference: package.validate().unwrap().resolve(
                Some(&Identity {
                    species: "Sampleton".into(),
                    form: Some("Normal".into()),
                }),
                None,
            ),
        }
    }

    #[test]
    fn defense_uses_external_bulk_and_multiplies_dual_typing() {
        let mut package = crate::gym::fictional_package();
        // Invented chart: electric->normal 1.6, electric->grass .625 cancel.
        let pokemon = pokemon(&package);
        let roles = RoleModel::new(Some(&package)).assess(&pokemon, None, &[]);
        let defense = &roles[0];
        assert_eq!(defense.state, "conditional_potential");
        assert!(defense
            .reasons
            .iter()
            .any(|reason| reason.contains("3 of 3")));
        assert!(!defense
            .reasons
            .iter()
            .any(|reason| reason.contains("vulnerable to incoming electric")));
        assert!(defense
            .missing
            .iter()
            .any(|reason| reason.contains("current level")));
        package
            .combat
            .as_mut()
            .unwrap()
            .effectiveness
            .get_mut("electric")
            .unwrap()
            .insert("grass".into(), 1.0);
        let roles = RoleModel::new(Some(&package)).assess(&pokemon, None, &[]);
        assert!(roles[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("vulnerable to incoming electric")));
    }

    #[test]
    fn rocket_type_signal_is_not_pve_dps_or_battle_readiness() {
        let package = crate::gym::fictional_package();
        let roles = RoleModel::new(Some(&package)).assess(&pokemon(&package), None, &[]);
        let rocket = &roles[1];
        assert_eq!(rocket.state, "conditional_type_coverage");
        assert!(rocket.summary.contains("fire types"));
        assert!(rocket
            .missing
            .iter()
            .any(|reason| reason.contains("turn timing")));
        assert!(!rocket.reasons.iter().any(|reason| reason.contains("DPS")));
        assert_eq!(roles[3].state, "retention_review");
    }

    #[test]
    fn unknown_identity_and_missing_optional_facts_never_become_disposal() {
        let package = crate::gym::fictional_package();
        let mut pokemon = pokemon(&package);
        pokemon.reference.state = ResolutionState::Unavailable;
        pokemon.ivs = None;
        let roles = RoleModel::new(Some(&package)).assess(&pokemon, None, &[]);
        assert_eq!(roles[0].state, "insufficient_data");
        assert_eq!(roles[1].state, "insufficient_data");
        assert_eq!(roles[3].state, "retention_review");
        pokemon.reference.state = ResolutionState::Resolved;
        pokemon.reference.dataset.version = "different-source-version".into();
        assert_eq!(
            RoleModel::new(Some(&package)).assess(&pokemon, None, &[])[0].state,
            "insufficient_data"
        );
    }

    #[test]
    fn stale_battle_facts_do_not_assert_readiness_or_remove_retention_safeguards() {
        let package = crate::gym::fictional_package();
        let details = DetailsView {
            values: crate::individual_details::IndividualDetails {
                protected: true,
                current_hp: Some(50),
                maximum_hp: Some(100),
                ..Default::default()
            },
            source: "manual",
            observation_revision: "1".into(),
            saved_at_revision: Some("1".into()),
            needs_reconfirmation: false,
            evidence_token: "fixture".into(),
        };
        let roles = RoleModel::new(Some(&package)).assess(&pokemon(&package), Some(&details), &[]);
        assert_eq!(roles[3].state, "retain_protected_value");
        assert!(roles[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("heal in the game")));
        let stale = DetailsView {
            needs_reconfirmation: true,
            ..details
        };
        let roles = RoleModel::new(Some(&package)).assess(&pokemon(&package), Some(&stale), &[]);
        assert_eq!(roles[3].state, "retain_protected_value");
        assert!(!roles[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("heal in the game")));
        assert!(roles
            .iter()
            .all(|role| role.missing.iter().any(|reason| reason.contains("predate"))));
    }

    #[test]
    fn league_over_cap_is_specific_and_does_not_assert_all_pvp_value_absent() {
        let package = crate::gym::fictional_package();
        let leagues = vec![LeagueAssessment {
            league: "Great League".into(),
            cp_limit: Some(1500),
            state: "ineligible_observed_cp".into(),
            readiness: crate::utility::Readiness::InsufficientData,
            potential: None,
            alternative_pokemon_id: None,
            explanation: "".into(),
        }];
        let roles = RoleModel::new(Some(&package)).assess(&pokemon(&package), None, &leagues);
        assert_eq!(roles[2].state, "insufficient_data");
        assert!(roles[2].reasons[0].contains("Great League"));
        assert_eq!(roles[3].state, "retention_review");
    }

    #[test]
    fn trainer_energy_options_use_supplied_turns_and_do_not_become_win_rates() {
        let mut package = crate::gym::fictional_package();
        package.trainer_combat = Some(crate::trainer_combat_reference::fictional_reference());
        let roles = RoleModel::new(Some(&package)).assess(&pokemon(&package), None, &[]);
        assert_eq!(roles[1].state, "trainer_move_potential");
        assert_eq!(roles[2].state, "move_kit_potential");
        // Independent arithmetic: 40/8=5 fast attacks; 5*2=10 turns;
        // energy/turn=8/2=4; power/energy=80/40=2.
        for role in [&roles[1], &roles[2]] {
            assert!(role
                .reasons
                .iter()
                .any(|reason| reason.contains("4.00 energy/turn")
                    && reason.contains("2.00 base power/energy")
                    && reason.contains("5 fast moves, 10 fast-action turns")));
            assert!(role
                .reasons
                .iter()
                .any(|reason| reason.contains("fire") && reason.contains("water")));
        }
        assert!(!roles[1]
            .missing
            .iter()
            .any(|missing| missing.starts_with("Trainer-Battle move power")));
        assert_eq!(roles[3].state, "retention_review");
    }

    #[test]
    fn tied_trainer_options_do_not_change_when_equivalent_source_order_changes() {
        let mut package = crate::gym::fictional_package();
        let mut trainer = crate::trainer_combat_reference::fictional_reference();
        let mut tied = trainer
            .moves
            .iter()
            .find(|m| m.kind == "fast")
            .unwrap()
            .clone();
        tied.id = "move-999999".into();
        tied.name = "Alternative Sample Tap".into();
        trainer.pools[0].fast_move_ids.push(tied.id.clone());
        trainer.moves.push(tied);
        package.trainer_combat = Some(trainer);
        let before = RoleModel::new(Some(&package)).assess(&pokemon(&package), None, &[]);
        let reference = package.trainer_combat.as_mut().unwrap();
        reference.moves.reverse();
        reference.pools[0].fast_move_ids.reverse();
        reference.pools[0].charged_move_ids.reverse();
        let after = RoleModel::new(Some(&package)).assess(&pokemon(&package), None, &[]);
        assert_eq!(
            before, after,
            "Equivalent move ordering must not change the chosen role recommendations"
        );
    }

    #[test]
    fn trainer_options_never_fabricate_zero_energy_gain_and_report_buffs() {
        let mut trainer = crate::trainer_combat_reference::fictional_reference();
        let fast_index = trainer
            .moves
            .iter()
            .position(|movement| movement.kind == "fast")
            .unwrap();
        trainer.moves[fast_index].energy_delta = 0;
        assert!(trainer_kits(&trainer, "dex-61001-normal").is_empty());
        trainer.moves[fast_index].energy_delta = 8;
        trainer.moves[fast_index].has_buffs = true;
        let mut package = crate::gym::fictional_package();
        package.trainer_combat = Some(trainer);
        let roles = RoleModel::new(Some(&package)).assess(&pokemon(&package), None, &[]);
        assert!(roles[1]
            .missing
            .iter()
            .any(|missing| missing.contains("buff/debuff")));
        assert!(roles[2]
            .missing
            .iter()
            .any(|missing| missing.contains("buff/debuff")));
    }

    #[test]
    fn defense_summaries_distinguish_external_bulk_without_claiming_battle_rank() {
        fn expanded_package(lower: usize) -> LocalPackage {
            let mut package = crate::gym::fictional_package();
            for index in 0..9 {
                let dex = 61100 + index;
                let mut species = package.catalog["species"][0].clone();
                species["id"] = format!("dex-{dex}").into();
                species["dexNumber"] = serde_json::json!(dex);
                species["name"] = format!("Fictionbulk{index}").into();
                species["aliases"] = serde_json::json!([]);
                package.catalog["species"]
                    .as_array_mut()
                    .unwrap()
                    .push(species);
                let mut form = package.catalog["forms"][0].clone();
                form["id"] = format!("dex-{dex}-normal").into();
                form["speciesId"] = format!("dex-{dex}").into();
                let base = if index < lower { 50 } else { 500 };
                form["baseStats"]["defense"] = serde_json::json!(base);
                form["baseStats"]["stamina"] = serde_json::json!(base);
                package.catalog["forms"].as_array_mut().unwrap().push(form);
            }
            package.validate().unwrap();
            package
        }
        for (lower, expected) in [
            (9, "Stronger natural-bulk"),
            (0, "Lower natural bulk"),
            (3, "Middle-range natural bulk"),
        ] {
            let package = expanded_package(lower);
            let roles = RoleModel::new(Some(&package)).assess(&pokemon(&package), None, &[]);
            assert!(roles[0].summary.starts_with(expected));
            assert!(roles[0]
                .reasons
                .iter()
                .any(|reason| reason.contains("not a defender battle ranking")));
            assert!(roles[0]
                .missing
                .iter()
                .any(|missing| missing.contains("placement eligibility")));
        }
        let package = crate::gym::fictional_package();
        assert!(
            RoleModel::new(Some(&package)).assess(&pokemon(&package), None, &[])[0]
                .summary
                .contains("small imported reference subset")
        );
    }
}
