//! Versioned multi-role suggestions. A suggested tag never mutates applied tags.
//! Criteria are contextual reference heuristics, not battle wins or meta ranks.
use crate::{
    gym::{GymAssessment, GymModel},
    individual_details::DetailsView,
    inventory::Pokemon,
    reference::{Catalog, Form, ResolutionState, Species},
    reference_store::LocalPackage,
    strategy::StrategyReport,
    trainer_combat_reference::{TrainerCombatReference, TrainerMove},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: &str = "individual-decision-1";
const MIN_BENCHMARK: usize = 10;
const STRONG_PERCENTILE: f64 = 75.0;
const MAX_FUTURE_FORMS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IndividualDecision {
    pub suggestions: Vec<SuggestedUse>,
    pub summary: String,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SuggestedUse {
    pub id: String,
    pub tag: String,
    pub state: String,
    pub target_form_id: Option<String>,
    pub target_name: Option<String>,
    pub explanation: String,
    pub evidence: Vec<String>,
    pub missing: Vec<String>,
}

fn suggestion(tag: &str, state: &str, target: &Pokemon, explanation: String) -> SuggestedUse {
    let form = target.reference.form.as_ref().map(|f| f.id.clone());
    SuggestedUse {
        id: format!(
            "{}:{}",
            tag.to_ascii_lowercase().replace([' ', '/'], "-"),
            form.as_deref().unwrap_or("unresolved")
        ),
        tag: tag.into(),
        state: state.into(),
        target_form_id: form,
        target_name: Some(
            target
                .reference
                .species
                .as_ref()
                .map_or_else(|| target.display_name.clone(), |s| s.name.clone()),
        ),
        explanation,
        evidence: vec![],
        missing: vec![],
    }
}

fn exceptional(form: &Form, dex: u16) -> bool {
    [132, 235, 292].contains(&dex)
        || ["mega", "primal"]
            .iter()
            .any(|s| form.name.to_ascii_lowercase().contains(s))
}

#[derive(Clone)]
struct EnergyKit {
    fast: String,
    charged: String,
    attack_type: String,
    fast_id: String,
    charged_id: String,
    fast_turns: u32,
    dpe: f64,
    dpt: f64,
    buffs: bool,
}

fn energy_kits(reference: &TrainerCombatReference, form: &str) -> Vec<EnergyKit> {
    let Some(pool) = reference.pools.iter().find(|p| p.form_id == form) else {
        return vec![];
    };
    let mut result = BTreeMap::<String, EnergyKit>::new();
    let valid = |m: &&TrainerMove| {
        m.power > 0.0
            && !["hidden power", "transform"].contains(&m.name.to_ascii_lowercase().as_str())
    };
    for fast in reference
        .moves
        .iter()
        .filter(|m| pool.fast_move_ids.contains(&m.id) && m.energy_delta > 0)
        .filter(valid)
    {
        for charged in reference
            .moves
            .iter()
            .filter(|m| pool.charged_move_ids.contains(&m.id) && m.energy_delta < 0)
            .filter(valid)
        {
            let kit = EnergyKit {
                fast: fast.name.clone(),
                charged: charged.name.clone(),
                attack_type: charged.move_type.clone(),
                fast_id: fast.id.clone(),
                charged_id: charged.id.clone(),
                fast_turns: u32::from(charged.energy_delta.unsigned_abs())
                    .div_ceil(fast.energy_delta as u32)
                    * u32::from(fast.turns),
                dpe: charged.power / -f64::from(charged.energy_delta),
                dpt: fast.power / f64::from(fast.turns),
                buffs: fast.has_buffs || charged.has_buffs,
            };
            let replace = result.get(&kit.attack_type).is_none_or(|old| {
                kit.fast_turns
                    .cmp(&old.fast_turns)
                    .then_with(|| old.dpe.total_cmp(&kit.dpe))
                    .then_with(|| old.dpt.total_cmp(&kit.dpt))
                    .then_with(|| {
                        (&kit.fast_id, &kit.charged_id).cmp(&(&old.fast_id, &old.charged_id))
                    })
                    .is_lt()
            });
            if replace {
                result.insert(kit.attack_type.clone(), kit);
            }
        }
    }
    result.into_values().collect()
}

/// Construct once per collection snapshot, alongside the existing role models.
pub struct DecisionModel<'a> {
    package: Option<&'a LocalPackage>,
    catalog: Option<Catalog>,
    gym: Option<GymModel<'a>>,
    forms: BTreeMap<String, (Species, Form)>,
    bulk: Vec<f64>,
    attack: Vec<f64>,
    kits: BTreeMap<String, Vec<EnergyKit>>,
    energy_bench: BTreeMap<String, Vec<EnergyKit>>,
}

impl<'a> DecisionModel<'a> {
    pub fn new(package: Option<&'a LocalPackage>) -> Self {
        let catalog = package.and_then(|p| p.validate().ok());
        let package = package.filter(|_| catalog.is_some());
        let gym = package.and_then(|p| GymModel::new(p).ok().flatten());
        let mut forms = BTreeMap::new();
        if let Some(p) = package {
            let species: BTreeMap<String, Species> = p.catalog["species"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|s| {
                    serde_json::from_value::<Species>(s.clone())
                        .ok()
                        .map(|s| (s.id.clone(), s))
                })
                .collect();
            for raw in p.catalog["forms"].as_array().unwrap() {
                if let Ok(form) = serde_json::from_value::<Form>(raw.clone()) {
                    if let Some(s) = species.get(&form.species_id) {
                        if form.base_stats.is_some() && !exceptional(&form, s.dex_number) {
                            forms.insert(form.id.clone(), (s.clone(), form));
                        }
                    }
                }
            }
        }
        let bulk = forms
            .values()
            .map(|(_, f)| {
                let s = f.base_stats.as_ref().unwrap();
                (f64::from(s.defense) + 15.0) * (f64::from(s.stamina) + 15.0)
            })
            .collect();
        let attack = forms
            .values()
            .map(|(_, f)| f64::from(f.base_stats.as_ref().unwrap().attack) + 15.0)
            .collect();
        let mut kits = BTreeMap::new();
        let mut energy_bench = BTreeMap::<String, Vec<EnergyKit>>::new();
        if let Some(reference) = package.and_then(|p| p.trainer_combat.as_ref()) {
            for form in forms.keys() {
                let options = energy_kits(reference, form);
                for kit in &options {
                    energy_bench
                        .entry(kit.attack_type.clone())
                        .or_default()
                        .push(kit.clone());
                }
                kits.insert(form.clone(), options);
            }
        }
        Self {
            package,
            catalog,
            gym,
            forms,
            bulk,
            attack,
            kits,
            energy_bench,
        }
    }

    fn supported_form(&self, pokemon: &Pokemon) -> Option<&Form> {
        let package = self.package?;
        if pokemon.reference.state != ResolutionState::Resolved
            || pokemon.reference.dataset.dataset_id
                != package.catalog["dataset"]["datasetId"].as_str()?
            || pokemon.reference.dataset.version
                != package.catalog["dataset"]["version"].as_str()?
        {
            return None;
        }
        self.forms
            .get(&pokemon.reference.form.as_ref()?.id)
            .map(|(_, f)| f)
    }

    fn fresh_current(&self, pokemon: &Pokemon, detail: Option<&DetailsView>) -> bool {
        let Some(detail) = detail.filter(|d| !d.needs_reconfirmation) else {
            return false;
        };
        let v = &detail.values;
        let Some(level) = v.level else {
            return false;
        };
        if v.shadow == Some(true)
            || !matches!((v.current_hp,v.maximum_hp),(Some(a),Some(b))if a>0&&a==b)
        {
            return false;
        }
        let Some((_, cpm)) = self
            .package
            .and_then(|p| p.cp_multipliers.iter().find(|(l, _)| *l == level))
        else {
            return false;
        };
        let (Some(iv), Some(base)) = (
            pokemon.ivs,
            self.supported_form(pokemon)
                .and_then(|f| f.base_stats.as_ref()),
        ) else {
            return false;
        };
        let cp = (((f64::from(base.attack) + f64::from(iv.attack))
            * (f64::from(base.defense) + f64::from(iv.defense)).sqrt()
            * (f64::from(base.stamina) + f64::from(iv.hp)).sqrt()
            * cpm.powi(2))
            / 10.0)
            .floor()
            .max(10.0) as i64;
        let hp = ((f64::from(base.stamina) + f64::from(iv.hp)) * cpm)
            .floor()
            .max(10.0) as u32;
        v.maximum_hp == Some(hp) && pokemon.cp.is_none_or(|observed| observed == cp)
    }

    fn attack_use(
        &self,
        target: &Pokemon,
        detail: Option<&DetailsView>,
        future: bool,
        provided: Option<&GymAssessment>,
    ) -> SuggestedUse {
        if self.supported_form(target).is_none()
            || detail.is_some_and(|d| d.values.shadow == Some(true))
        {
            let mut use_=suggestion("ATTACKER","needs_evidence",target,"Current attacker strength is unresolved for this identity or variant; ordinary reference damage is not applied to unsupported mechanics.".into());
            use_.missing.push(
                "Supported active-reference identity and applicable variant mechanics".into(),
            );
            return use_;
        }
        let assessed;
        let gym = if let Some(gym) = provided {
            Some(gym)
        } else {
            assessed = self.gym.as_ref().map(|m| m.assess(target, detail));
            assessed.as_ref()
        };
        let qualified = gym
            .filter(|_| !detail.is_some_and(|d| d.values.shadow == Some(true)))
            .and_then(|g| {
                g.builds.iter().find(|b| {
                    b.compared_forms >= MIN_BENCHMARK && b.reference_percentile >= STRONG_PERCENTILE
                })
            });
        let current = qualified.is_some_and(|b| b.move_state == "confirmed")
            && !future
            && self.fresh_current(target, detail);
        let mut use_ = suggestion(
            "ATTACKER",
            if current {
                "supported_current"
            } else if qualified.is_some() {
                "preparation_potential"
            } else {
                "needs_evidence"
            },
            target,
            if let Some(build) = qualified {
                format!("{} {} attack option against {} types. Its supported set is in the upper quarter of the imported same-type neutral cycle benchmark; this is contextual potential, not a win-rate.",if future{"Future-form"}else if current{"Current-form"}else{"Current-form preparation"},build.attack_type,if build.strong_against.is_empty(){"no established favorable".into()}else{build.strong_against.join(", ")})
            } else {
                "A strong general attacker suggestion is not established by the available same-type reference comparison. Specific opponent contexts may still make this individual useful.".into()
            },
        );
        if let Some(gym) = gym {
            for b in gym.builds.iter().take(3) {
                use_.evidence.push(format!(
                    "{}: {} + {}; {:.1}th reference percentile among {} supported forms.",
                    b.attack_type,
                    b.fast_move,
                    b.charged_move,
                    b.reference_percentile,
                    b.compared_forms
                ));
            }
            if qualified.is_none() {
                use_.missing.push(format!("Policy needs at least {MIN_BENCHMARK} same-type reference forms and percentile ≥{STRONG_PERCENTILE}; a small or weaker comparison is not a strong-role verdict."));
            }
            use_.missing.extend(gym.missing.clone());
        } else {
            use_.missing
                .push("Resolved identity and complete compatible gym combat reference".into());
        }
        if !current {
            use_.missing.push(if future{"Evolution, move acquisition and fresh observations after the real change"}else{"Fresh current level, matching observed attacks and HP before describing a current-use option"}.into());
        }
        use_
    }

    fn defense_use(&self, target: &Pokemon, future: bool, shadow: bool) -> SuggestedUse {
        let mut use_=suggestion("GYM DEF","needs_evidence",target,"Defender suitability needs supported natural bulk, typing and placement context; another owned specimen alone does not decide it.".into());
        let Some(form) = self.supported_form(target) else {
            use_.missing
                .push("Supported ordinary-form identity and reference stats".into());
            return use_;
        };
        let base = form.base_stats.as_ref().unwrap();
        let bulk = (f64::from(base.defense) + target.ivs.map_or(15.0, |v| f64::from(v.defense)))
            * (f64::from(base.stamina) + target.ivs.map_or(15.0, |v| f64::from(v.hp)));
        let percentile = 100.0 * self.bulk.iter().filter(|b| **b <= bulk).count() as f64
            / self.bulk.len().max(1) as f64;
        use_.evidence.push(format!("Equal-level Defense × Stamina proxy at percentile {percentile:.1} among {} imported forms, using actual IVs when known and hypothetical 15 otherwise.",self.bulk.len()));
        if let (Some(combat), Some(types)) = (
            self.package.and_then(|p| p.combat.as_ref()),
            form.types.as_ref(),
        ) {
            let own = types
                .iter()
                .map(|t| format!("{t:?}").to_ascii_lowercase())
                .collect::<Vec<_>>();
            let multiplier = |attack: &str| {
                own.iter()
                    .map(|kind| combat.effectiveness[attack][kind])
                    .product::<f64>()
            };
            let resisted = crate::combat_reference::TYPES
                .iter()
                .filter(|t| multiplier(t) < 1.0)
                .copied()
                .collect::<Vec<_>>();
            let weak = crate::combat_reference::TYPES
                .iter()
                .filter(|t| multiplier(t) > 1.0)
                .copied()
                .collect::<Vec<_>>();
            if !resisted.is_empty() {
                use_.evidence.push(format!(
                    "Typing resists incoming {} attacks in the imported chart.",
                    resisted.join(", ")
                ));
            }
            if !weak.is_empty() {
                use_.evidence.push(format!("Typing is vulnerable to incoming {} attacks; actual opponent moves and dual-type effects matter.",weak.join(", ")));
            }
        }
        if self.bulk.len() >= MIN_BENCHMARK
            && percentile >= STRONG_PERCENTILE
            && !shadow
            && form.types.is_some()
            && self.package.is_some_and(|p| p.combat.is_some())
        {
            use_.state = "preparation_potential".into();
            use_.explanation=format!("{} defender candidate with upper-quarter natural bulk in the imported ordinary-form reference. Review typing, current HP and actual gym placement eligibility.",if future{"Future-form"}else{"Current-form"});
        } else {
            use_.missing.push("A sufficiently large upper-quarter ordinary bulk comparison, supported typing and variant mechanics".into());
        }
        use_.missing.push("Gym placement eligibility, motivation, opponent attacks and defender timing remain unverified; this is not a time-to-defeat ranking".into());
        use_
    }

    fn rocket_use(
        &self,
        target: &Pokemon,
        detail: Option<&DetailsView>,
        future: bool,
    ) -> SuggestedUse {
        let mut use_=suggestion("ROCKET","needs_evidence",target,"A broad Rocket recommendation needs Trainer-Battle energy access, usable attack coverage and supported stats. Gym-cycle output does not establish it.".into());
        let Some(form) = self.supported_form(target) else {
            return use_;
        };
        let Some(combat) = self.package.and_then(|p| p.combat.as_ref()) else {
            use_.missing.push("Type-effectiveness reference".into());
            return use_;
        };
        let base = form.base_stats.as_ref().unwrap();
        let a = f64::from(base.attack) + target.ivs.map_or(15.0, |v| f64::from(v.attack));
        let bulk = (f64::from(base.defense) + 15.0) * (f64::from(base.stamina) + 15.0);
        let attack_support =
            self.attack.iter().filter(|x| **x <= a).count() * 2 >= self.attack.len();
        let bulk_support = self.bulk.iter().filter(|x| **x <= bulk).count() * 2 >= self.bulk.len();
        let mut qualifying = Vec::new();
        for kit in self.kits.get(&form.id).into_iter().flatten() {
            let benchmark = &self.energy_bench[&kit.attack_type];
            let energy_rank = 100.0
                * benchmark
                    .iter()
                    .filter(|b| b.fast_turns >= kit.fast_turns)
                    .count() as f64
                / benchmark.len() as f64;
            let efficient =
                benchmark.iter().filter(|b| b.dpe <= kit.dpe).count() * 2 >= benchmark.len();
            let favorable: Vec<_> = combat.effectiveness[&kit.attack_type]
                .iter()
                .filter(|(_, effect)| **effect > 1.0)
                .map(|(kind, _)| kind.clone())
                .collect();
            if benchmark.len() >= MIN_BENCHMARK
                && energy_rank >= STRONG_PERCENTILE
                && efficient
                && (attack_support || bulk_support)
                && !favorable.is_empty()
            {
                qualifying.push((kit, energy_rank, favorable));
            }
        }
        for (kit, rank, favorable) in qualifying.iter().take(3) {
            use_.evidence.push(format!("{} energy-oriented kit {} + {}: {} fast-action turns to first charge; access percentile {rank:.1} among {} same-type options; favorable charged typing against {}.",kit.attack_type,kit.fast,kit.charged,kit.fast_turns,self.energy_bench[&kit.attack_type].len(),favorable.join(", ")));
        }
        if !qualifying.is_empty() {
            let observed = detail
                .filter(|d| !d.needs_reconfirmation)
                .map(|d| &d.values);
            let matches = qualifying.iter().any(|(k, _, _)| {
                !k.buffs
                    && observed.is_some_and(|v| {
                        v.fast_move
                            .as_ref()
                            .is_some_and(|m| m.eq_ignore_ascii_case(&k.fast))
                            && [&v.charged_move, &v.second_charged_move].iter().any(|m| {
                                m.as_ref()
                                    .is_some_and(|m| m.eq_ignore_ascii_case(&k.charged))
                            })
                    })
            });
            use_.state = if !future && matches && self.fresh_current(target, detail) {
                "supported_current"
            } else {
                "preparation_potential"
            }
            .into();
            use_.explanation=format!("{} Rocket option with comparatively early charged-attack access and favorable type coverage. This energy-oriented role does not predict a win or shield outcome.",if future{"Future-form preparation"}else{"Current-form"});
        } else {
            use_.missing.push("Policy requires ≥10 same-type options, upper-quarter charge access, at least median charged efficiency and supported attack/bulk; available evidence does not establish this combination".into());
        }
        use_.missing.push("Actual Rocket lineup, shields, opponent attacks and applicable variant/buff mechanics; proposed moves require acquisition and confirmation".into());
        use_
    }

    fn future_forms(&self, pokemon: &Pokemon) -> Vec<Pokemon> {
        let Some(catalog) = &self.catalog else {
            return vec![];
        };
        let mut queue = vec![(pokemon.clone(), 0)];
        let mut seen = BTreeSet::new();
        let mut result = Vec::new();
        if let Some(f) = &pokemon.reference.form {
            seen.insert(f.id.clone());
        }
        let mut index = 0;
        while index < queue.len() && result.len() < MAX_FUTURE_FORMS {
            let (source, depth) = queue[index].clone();
            index += 1;
            if depth >= 3 {
                continue;
            }
            let mut edges = catalog.evolution_targets(&source.reference);
            edges.sort_by(|a, b| a.1.id.cmp(&b.1.id));
            for (species, form) in edges {
                if result.len() >= MAX_FUTURE_FORMS {
                    break;
                }
                if !seen.insert(form.id.clone()) {
                    continue;
                }
                let mut next = pokemon.clone();
                next.display_name = species.name.clone();
                next.cp = None;
                next.reference.species = Some(species);
                next.reference.form = Some(form);
                queue.push((next.clone(), depth + 1));
                result.push(next);
            }
        }
        result
    }

    pub fn assess(
        &self,
        report: &StrategyReport,
        pokemon: &Pokemon,
        detail: Option<&DetailsView>,
    ) -> IndividualDecision {
        let shadow = detail.is_some_and(|d| d.values.shadow == Some(true));
        let mut suggestions = vec![
            self.attack_use(pokemon, detail, false, report.gym.as_ref()),
            self.defense_use(pokemon, false, shadow),
            self.rocket_use(pokemon, detail, false),
        ];
        let futures = self.future_forms(pokemon);
        for future in &futures {
            let uses = [
                self.attack_use(future, detail, true, None),
                self.defense_use(future, true, shadow),
                self.rocket_use(future, detail, true),
            ];
            suggestions.extend(uses.into_iter().filter(|u|u.state=="preparation_potential").map(|mut u|{u.missing.push("Evolution eligibility and current release/event availability need verification; future potential is not evidence of obtainability".into());u}));
        }
        for (index, tag) in ["PVP 1500", "PVP 2500"].into_iter().enumerate() {
            let mut use_=suggestion(tag,"needs_evidence",pokemon,"Competitive league usefulness remains unverified. IV/stat-product rank and move efficiency alone do not justify a strong PvP tag.".into());
            if let Some(league) = report.leagues.get(index) {
                use_.evidence.push(league.explanation.clone());
                if let Some(p) = &league.potential {
                    use_.evidence.push(format!("Theoretical target level {}, CP {}, IV rank {}/{} within this form and imported ceiling {}.",p.level,p.cp,p.iv_rank,p.ranked_spreads,p.level_ceiling));
                }
            }
            if !futures.is_empty() {
                use_.evidence.push(format!("{} reachable future form(s) also require league-specific competitive assessment; current-form uncertainty is not evidence of no future PvP value.",futures.len()));
            }
            use_.missing.push("Compatible current competitive reference/matchup evidence, legal league/cup context and confirmed intended moves".into());
            suggestions.push(use_);
        }
        let special = report.protected
            || detail.is_some_and(|d| {
                d.values.protected
                    || d.values.shiny == Some(true)
                    || d.values.favorite == Some(true)
                    || d.values.shadow == Some(true)
                    || d.values.purified == Some(true)
            });
        let mut collection=suggestion(if special{"SAVE/SPECIAL"}else{"KEEP"},"collection_value",pokemon,
            if special{"Retain this protected or special individual independently of combat recommendations."}else if report.unique_form==Some(true){"Keep this individual's currently unique confirmed form for collection value; combat usefulness is assessed independently."}else{"Keep pending contribution review. Several individuals of the same species or role can remain useful; duplicate status is not a disposal rule."}.into());
        if let Some(d) = detail {
            for (known, label) in [
                (d.values.shiny == Some(true), "Confirmed Shiny"),
                (d.values.favorite == Some(true), "Confirmed favorite"),
                (d.values.protected, "Explicit protection"),
                (d.values.shadow == Some(true), "Confirmed Shadow"),
                (d.values.purified == Some(true), "Confirmed Purified"),
            ] {
                if known {
                    collection.evidence.push(label.into());
                }
            }
            if d.values.weight_kg.is_some() || d.values.height_m.is_some() {
                collection.missing.push("Weight/height are recorded observations; species/form size distributions or explicit special-size evidence are needed before a rarity claim".into());
            }
        }
        collection.evidence.push(format!("{} other confirmed individuals of this form; this count does not limit how many may be useful.",report.duplicate_count));
        suggestions.push(collection);
        let mut transfer=suggestion("TRANSFER","retention_review",pokemon,"Retain for review. The evidence does not establish that this individual has lost its contribution across current uses, future preparation and collection value.".into());
        if special {
            transfer.explanation="Retain this protected or special individual. Comparative combat potential does not override its preservation reasons.".into();
        }
        transfer.evidence.push("A stronger owned alternative or promising newcomer changes investment priority without automatically retiring an existing useful individual.".into());
        transfer.missing.push("Collection-wide contribution, team depth/reserves, readiness of replacements and personal preservation review before any in-game transfer".into());
        suggestions.push(transfer);
        for use_ in &mut suggestions {
            use_.evidence.sort();
            use_.evidence.dedup();
            use_.missing.sort();
            use_.missing.dedup();
        }
        suggestions.sort_by(|a, b| {
            a.tag
                .cmp(&b.tag)
                .then_with(|| a.target_form_id.cmp(&b.target_form_id))
                .then_with(|| a.id.cmp(&b.id))
        });
        let recommended = suggestions
            .iter()
            .filter(|s| {
                matches!(
                    s.state.as_str(),
                    "supported_current" | "preparation_potential"
                )
            })
            .count();
        let mut missing=vec!["These suggestions use declared contextual reference heuristics, not a universal quality score or competitive meta ranking.".into()];
        if self.supported_form(pokemon).is_none() {
            missing.push("Confirm an identity that belongs to the active compatible reference before assigning battle uses.".into());
        }
        if detail.is_some_and(|d| d.needs_reconfirmation) {
            missing.push("Reconfirm optional observations invalidated by a real action/correction; preservation facts remain protected.".into());
        }
        IndividualDecision {
            summary: if recommended > 0 {
                format!("{recommended} supported current-form or preparation role option(s). Review current versus future form and conditions; several uses and several owned individuals can coexist.")
            } else {
                "No strong combat role is established by the available evidence. Preserve collection value and review missing context or future opportunities before investment or disposal.".into()
            },
            suggestions,
            missing,
        }
    }
}

pub fn assess(
    report: &StrategyReport,
    pokemon: &Pokemon,
    detail: Option<&DetailsView>,
    package: Option<&LocalPackage>,
) -> IndividualDecision {
    DecisionModel::new(package).assess(report, pokemon, detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        individual_details::IndividualDetails,
        inventory::{AppraisalIvs, PokemonId},
        reference::Identity,
    };

    fn package(strong_competitors: bool) -> LocalPackage {
        let mut package = crate::gym::fictional_package();
        let mut trainer = crate::trainer_combat_reference::fictional_reference();
        let base_trainer_pool = trainer.pools[0].clone();
        for dex in [61002, 61003] {
            let mut p = base_trainer_pool.clone();
            p.form_id = format!("dex-{dex}-normal");
            trainer.pools.push(p);
        }
        for index in 0..11 {
            let dex = 61100 + index;
            let id = format!("dex-{dex}");
            let mut s = package.catalog["species"][0].clone();
            s["id"] = id.clone().into();
            s["dexNumber"] = serde_json::json!(dex);
            s["name"] = format!("Fictioncompetitor{index}").into();
            s["aliases"] = serde_json::json!([]);
            package.catalog["species"].as_array_mut().unwrap().push(s);
            let mut f = package.catalog["forms"][0].clone();
            f["id"] = format!("{id}-normal").into();
            f["speciesId"] = id.clone().into();
            let stat = if strong_competitors { 500 } else { 20 };
            f["baseStats"] = serde_json::json!({"attack":stat,"defense":stat,"stamina":stat});
            package.catalog["forms"].as_array_mut().unwrap().push(f);
            let mut p = package.combat.as_ref().unwrap().pools[0].clone();
            p.form_id = format!("{id}-normal");
            package.combat.as_mut().unwrap().pools.push(p);
            let mut p = base_trainer_pool.clone();
            p.form_id = format!("{id}-normal");
            trainer.pools.push(p);
        }
        package.trainer_combat = Some(trainer);
        package.cp_multipliers = (2..=80)
            .map(|half| {
                let level = f64::from(half) / 2.0;
                (level, 0.1 + level / 100.0)
            })
            .collect();
        package.validate().unwrap();
        package
    }
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
            created_at: "fictional".into(),
            updated_at: "fictional".into(),
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
    fn details() -> DetailsView {
        DetailsView {
            values: IndividualDetails {
                level: Some(30.0),
                level_evidence: Some("Explicit fictional manual level".into()),
                fast_move: Some("Sample Tap".into()),
                charged_move: Some("Sample Slam".into()),
                current_hp: Some(46),
                maximum_hp: Some(46),
                ..Default::default()
            },
            source: "manually_confirmed",
            observation_revision: "0".into(),
            saved_at_revision: Some("0".into()),
            needs_reconfirmation: false,
            evidence_token: "fictional".into(),
        }
    }
    fn report(
        package: &LocalPackage,
        pokemon: &Pokemon,
        detail: Option<&DetailsView>,
    ) -> StrategyReport {
        let mut report = crate::strategy::assess(&[(pokemon.clone(), vec![])], None)
            .unwrap()
            .remove(0);
        report.gym = GymModel::new(package)
            .unwrap()
            .map(|m| m.assess(pokemon, detail));
        report
    }
    fn use_<'a>(decision: &'a IndividualDecision, tag: &str, form: &str) -> &'a SuggestedUse {
        decision
            .suggestions
            .iter()
            .find(|s| s.tag == tag && s.target_form_id.as_deref() == Some(form))
            .unwrap()
    }

    #[test]
    fn recommends_multiple_justified_roles_without_requiring_a_selected_goal() {
        let package = package(false);
        let pokemon = pokemon(&package);
        let details = details();
        let report = report(&package, &pokemon, Some(&details));
        let decision = assess(&report, &pokemon, Some(&details), Some(&package));
        assert_eq!(
            use_(&decision, "ATTACKER", "dex-61001-normal").state,
            "supported_current"
        );
        assert_eq!(
            use_(&decision, "GYM DEF", "dex-61001-normal").state,
            "preparation_potential"
        );
        assert_eq!(
            use_(&decision, "ROCKET", "dex-61001-normal").state,
            "supported_current"
        );
        assert!(decision
            .suggestions
            .iter()
            .filter(|s| s.tag.starts_with("PVP"))
            .all(|s| s.state == "needs_evidence"));
        assert_eq!(
            use_(&decision, "TRANSFER", "dex-61001-normal").state,
            "retention_review"
        );
    }

    #[test]
    fn weak_reference_performance_and_small_samples_do_not_become_strong_role_tags() {
        for package in [package(true), crate::gym::fictional_package()] {
            let pokemon = pokemon(&package);
            let report = report(&package, &pokemon, None);
            let decision = assess(&report, &pokemon, None, Some(&package));
            assert!(decision
                .suggestions
                .iter()
                .filter(|s| matches!(s.tag.as_str(), "ATTACKER" | "GYM DEF" | "ROCKET"))
                .all(|s| s.state == "needs_evidence"));
            assert!(decision.suggestions.iter().any(|s| s.tag == "KEEP"));
        }
    }

    #[test]
    fn future_role_tags_name_the_evolution_and_do_not_pretend_the_current_form_is_ready() {
        let mut package = package(false);
        package.catalog["forms"][0]["baseStats"] =
            serde_json::json!({"attack":10,"defense":10,"stamina":10});
        package.catalog["forms"][0]["evolutions"] = serde_json::json!([{"targetFormId":"dex-61002-normal","requirements":["Fictional evolution condition"]}]);
        let pokemon = pokemon(&package);
        let report = report(&package, &pokemon, None);
        let decision = assess(&report, &pokemon, None, Some(&package));
        assert_eq!(
            use_(&decision, "ATTACKER", "dex-61001-normal").state,
            "needs_evidence"
        );
        let evolved = use_(&decision, "ATTACKER", "dex-61002-normal");
        assert_eq!(evolved.state, "preparation_potential");
        assert_eq!(evolved.target_name.as_deref(), Some("Sampleprime"));
        assert!(evolved
            .missing
            .iter()
            .any(|s| s.contains("after the real change")));
    }

    #[test]
    fn protection_survives_stale_observations_and_size_does_not_invent_rarity() {
        let package = package(false);
        let pokemon = pokemon(&package);
        let mut details = details();
        details.values.shiny = Some(true);
        details.values.protected = true;
        details.values.weight_kg = Some(999.0);
        details.needs_reconfirmation = true;
        let mut report = report(&package, &pokemon, Some(&details));
        report.protected = true;
        let decision = assess(&report, &pokemon, Some(&details), Some(&package));
        let keep = use_(&decision, "SAVE/SPECIAL", "dex-61001-normal");
        assert_eq!(keep.state, "collection_value");
        assert!(keep.evidence.contains(&"Confirmed Shiny".into()));
        assert!(keep.missing.iter().any(|s| s.contains("rarity claim")));
        assert!(!decision
            .suggestions
            .iter()
            .any(|s| s.state == "supported_current"));
    }

    #[test]
    fn useful_duplicates_and_owned_alternatives_do_not_trigger_automatic_retirement() {
        let package = package(false);
        let pokemon = pokemon(&package);
        let details = details();
        let mut report = report(&package, &pokemon, Some(&details));
        report.unique_form = Some(false);
        report.duplicate_count = 3;
        report.gym.as_mut().unwrap().owned_alternative_pokemon_id =
            Some("00000000-0000-4000-8000-000000000002".into());
        let decision = assess(&report, &pokemon, Some(&details), Some(&package));
        assert_eq!(
            use_(&decision, "ATTACKER", "dex-61001-normal").state,
            "supported_current"
        );
        assert_eq!(
            use_(&decision, "TRANSFER", "dex-61001-normal").state,
            "retention_review"
        );
        assert!(use_(&decision, "KEEP", "dex-61001-normal")
            .explanation
            .contains("Several individuals"));
    }

    #[test]
    fn repeated_and_reordered_equivalent_references_produce_identical_suggestions() {
        let package = package(false);
        let pokemon = pokemon(&package);
        let details = details();
        let report = report(&package, &pokemon, Some(&details));
        let baseline = assess(&report, &pokemon, Some(&details), Some(&package));
        for _ in 0..5 {
            assert_eq!(
                baseline,
                assess(&report, &pokemon, Some(&details), Some(&package))
            );
        }
        let mut reordered = package.clone();
        reordered.catalog["forms"].as_array_mut().unwrap().reverse();
        reordered.catalog["species"]
            .as_array_mut()
            .unwrap()
            .reverse();
        let combat = reordered.combat.as_mut().unwrap();
        combat.moves.reverse();
        combat.pools.reverse();
        let trainer = reordered.trainer_combat.as_mut().unwrap();
        trainer.moves.reverse();
        trainer.pools.reverse();
        assert_eq!(
            baseline,
            assess(&report, &pokemon, Some(&details), Some(&reordered))
        );
    }

    #[test]
    fn known_shadow_never_uses_ordinary_current_attacker_projection_even_when_stale() {
        let package = package(false);
        let pokemon = pokemon(&package);
        let mut details = details();
        details.values.shadow = Some(true);
        details.needs_reconfirmation = true;
        let report = report(&package, &pokemon, Some(&details));
        let decision = assess(&report, &pokemon, Some(&details), Some(&package));
        assert_eq!(
            use_(&decision, "ATTACKER", "dex-61001-normal").state,
            "needs_evidence"
        );
    }
}
