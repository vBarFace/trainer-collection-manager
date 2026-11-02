//! Deterministic individual preparation alternatives. A projection is never a game observation.
use crate::{
    combat_reference::{CombatMove, CombatReference},
    gym::GymModel,
    individual_details::DetailsView,
    inventory::Pokemon,
    preparation_resources::ResourceBudget,
    reference::{BaseStats, Form, PokemonType},
    reference_store::LocalPackage,
};
use serde::{Deserialize, Serialize};

pub const VERSION: &str = "preparation-2";
const MAX_PATHS: usize = 16;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceCost {
    pub candy: Option<u32>,
    pub candy_xl: Option<u32>,
    pub stardust: Option<u32>,
    pub items: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreparationAction {
    pub kind: String,
    pub description: String,
    pub requirements: Vec<String>,
    pub acquisition: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreparationPath {
    pub id: String,
    pub target_form_id: String,
    pub target_name: String,
    pub target_level: Option<f64>,
    pub role: String,
    pub actions: Vec<PreparationAction>,
    pub cost: ResourceCost,
    pub benefit: String,
    pub blockers: Vec<String>,
    pub affordability: String,
    pub executable: bool,
    #[serde(default)]
    pub future_roles: Vec<crate::roles::RoleAssessment>,
    #[serde(default)]
    pub served_roles: Vec<String>,
    #[serde(default)]
    pub projected_cp: Option<u32>,
    #[serde(default)]
    pub improvement_percent: Option<f64>,
    #[serde(default)]
    pub target_kind: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreparationPlan {
    pub state: String,
    pub summary: String,
    pub current_level: Option<f64>,
    pub paths: Vec<PreparationPath>,
    pub missing: Vec<String>,
    pub limitations: Vec<String>,
    pub resources: ResourceBudget,
    pub selected_path_id: Option<String>,
    #[serde(default)]
    pub primary_path_id: Option<String>,
    #[serde(default)]
    pub planning_mode: Option<String>,
    #[serde(default)]
    pub conflicts: Vec<crate::integrated_planning::PlanConflict>,
}

fn type_key(kind: &PokemonType) -> String {
    format!("{kind:?}").to_ascii_lowercase()
}
pub(crate) fn cpm(package: &LocalPackage, level: f64) -> Option<f64> {
    package
        .cp_multipliers
        .iter()
        .find(|(l, _)| *l == level)
        .map(|(_, m)| *m)
}
fn pair<'a>(
    combat: &'a CombatReference,
    fast: &str,
    charged: &str,
) -> Option<(&'a CombatMove, &'a CombatMove)> {
    let mut fasts = combat
        .moves
        .iter()
        .filter(|m| m.kind == "fast" && m.name.eq_ignore_ascii_case(fast));
    let mut charges = combat
        .moves
        .iter()
        .filter(|m| m.kind == "charged" && m.name.eq_ignore_ascii_case(charged));
    let f = fasts.next()?;
    let c = charges.next()?;
    (fasts.next().is_none()
        && charges.next().is_none()
        && f.energy_delta > 0
        && c.energy_delta < 0
        && f.power > 0.0
        && c.power > 0.0)
        .then_some((f, c))
}
fn cycle(
    base: &BaseStats,
    iv: u8,
    types: &[PokemonType],
    moves: (&CombatMove, &CombatMove),
    multiplier: f64,
    defender_multiplier: f64,
) -> f64 {
    let damage = |m: &CombatMove| {
        (m.power * (f64::from(base.attack) + f64::from(iv)) * multiplier
            / (2.0 * 215.0 * defender_multiplier)
            * if types.iter().any(|t| type_key(t) == m.move_type) {
                1.2
            } else {
                1.0
            })
        .floor()
            + 1.0
    };
    let n = (-f64::from(moves.1.energy_delta) / f64::from(moves.0.energy_delta)).ceil();
    (n * damage(moves.0) + damage(moves.1)) * 1000.0
        / (n * f64::from(moves.0.duration_ms) + f64::from(moves.1.duration_ms))
}
pub(crate) fn cp(pokemon: &Pokemon, form: &Form, multiplier: f64) -> Option<u32> {
    let base = form.base_stats.as_ref()?;
    let iv = pokemon.ivs?;
    Some(
        (((f64::from(base.attack) + f64::from(iv.attack))
            * (f64::from(base.defense) + f64::from(iv.defense)).sqrt()
            * (f64::from(base.stamina) + f64::from(iv.hp)).sqrt()
            * multiplier.powi(2))
            / 10.0)
            .floor()
            .max(10.0) as u32,
    )
}

fn form_name(package: &LocalPackage, id: &str) -> String {
    package.catalog["forms"]
        .as_array()
        .and_then(|forms| forms.iter().find(|f| f["id"].as_str() == Some(id)))
        .and_then(|form| {
            package.catalog["species"]
                .as_array()?
                .iter()
                .find(|s| s["id"] == form["speciesId"])
        })
        .and_then(|species| species["name"].as_str())
        .unwrap_or("the source-confirmed form")
        .to_owned()
}

/// Sum explicit half-level rows. A gap is unknown, never zero or interpolated.
pub fn power_up_cost(
    reference: &crate::preparation_reference::PreparationReference,
    from: f64,
    to: f64,
) -> Option<(u32, u32, u32)> {
    if !from.is_finite()
        || !to.is_finite()
        || !(1.0..=50.0).contains(&from)
        || to < from
        || to > 50.0
        || (from * 2.0).fract() != 0.0
        || (to * 2.0).fract() != 0.0
    {
        return None;
    }
    let mut cost = (0u32, 0u32, 0u32);
    let mut level = from;
    while level < to {
        let row = reference
            .power_up_costs
            .iter()
            .find(|row| row.from_level == level && row.to_level == level + 0.5)?;
        cost.0 = cost.0.checked_add(row.candy)?;
        cost.1 = cost.1.checked_add(row.candy_xl)?;
        cost.2 = cost.2.checked_add(row.stardust)?;
        level += 0.5;
    }
    Some(cost)
}

fn affordability(cost: &ResourceCost, budget: &ResourceBudget) -> String {
    let pairs = [
        (cost.candy, budget.values.candy),
        (cost.candy_xl, budget.values.candy_xl),
        (cost.stardust, budget.values.stardust),
    ];
    if pairs
        .iter()
        .any(|(c, b)| matches!((c,b),(Some(c),Some(b)) if c>b))
    {
        return "over_budget".into();
    }
    if !cost.items.is_empty()
        || pairs
            .iter()
            .any(|(c, b)| c.is_none() || (c != &Some(0) && b.is_none()))
    {
        "unknown"
    } else {
        "within_budget"
    }
    .into()
}

fn acquisition(
    name: &str,
    fast: bool,
    access: Option<&crate::preparation_reference::MoveAccess>,
) -> String {
    let Some(access) = access else {
        return format!("{name}: acquisition source missing; do not assume a normal TM works.");
    };
    let (normal, elite) = if fast {
        (&access.fast_moves, &access.elite_fast_moves)
    } else {
        (&access.charged_moves, &access.elite_charged_moves)
    };
    if normal.iter().any(|n| n.eq_ignore_ascii_case(name)) {
        format!("{name}: {} TM from the current ordinary move pool. Results are random; required attempts are unknown. Evolution can reroll attacks; confirm them afterwards.",if fast{"Fast"}else{"Charged"})
    } else if elite.iter().any(|n| n.eq_ignore_ascii_case(name)) {
        format!("{name}: listed as an Elite {} TM option; a normal TM is not established. Check current event/evolution availability and the in-game selection before spending.",if fast{"Fast"}else{"Charged"})
    } else {
        format!("{name}: absent from ordinary and Elite lists; acquisition is unknown. Wait for a verified event or updated source.")
    }
}

struct Build {
    attack_type: String,
    fast_move: String,
    charged_move: String,
    reference_summary: String,
    strong_against: Vec<String>,
}
fn trainer_build(package: &LocalPackage, form: &Form) -> Option<Build> {
    let reference = package.trainer_combat.as_ref()?;
    let pool = reference.pools.iter().find(|p| p.form_id == form.id)?;
    let mut pairs = Vec::new();
    for fast in reference.moves.iter().filter(|m| {
        pool.fast_move_ids.contains(&m.id)
            && m.energy_delta > 0
            && m.power > 0.0
            && !m.name.eq_ignore_ascii_case("Hidden Power")
    }) {
        for charged in reference.moves.iter().filter(|m| {
            pool.charged_move_ids.contains(&m.id) && m.energy_delta < 0 && m.power > 0.0
        }) {
            let turns = (-f64::from(charged.energy_delta) / f64::from(fast.energy_delta)).ceil()
                * f64::from(fast.turns);
            pairs.push((
                turns,
                -charged.power / -f64::from(charged.energy_delta),
                fast,
                charged,
            ));
        }
    }
    pairs.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then_with(|| a.1.total_cmp(&b.1))
            .then_with(|| a.2.name.cmp(&b.2.name))
            .then_with(|| a.3.name.cmp(&b.3.name))
    });
    let (turns, _, fast, charged) = pairs.first()?;
    let strong_against = package.combat.as_ref().map_or_else(Vec::new, |c| {
        c.effectiveness[&charged.move_type]
            .iter()
            .filter(|(_, m)| **m > 1.0)
            .map(|(t, _)| t.clone())
            .collect()
    });
    Some(Build{attack_type:charged.move_type.clone(),fast_move:fast.name.clone(),charged_move:charged.name.clone(),reference_summary:format!("Energy-oriented Trainer-Battle kit: {turns} fast-action turns to first charge from zero. Shield behavior, buffs, incoming attacks and actual matchups are not simulated."),strong_against})
}

fn league_level(pokemon: &Pokemon, form: &Form, package: &LocalPackage, limit: u32) -> Option<f64> {
    package
        .cp_multipliers
        .iter()
        .filter(|(l, _)| *l <= 50.0)
        .filter(|(_, m)| cp(pokemon, form, *m).is_some_and(|cp| cp <= limit))
        .map(|(l, _)| *l)
        .next_back()
}

// Explicit preparation heuristic: 90% of this form's supported level-40 model,
// not an opponent breakpoint. Never interpolate unsupported multipliers.
fn model_checkpoint(
    pokemon: &Pokemon,
    form: &Form,
    package: &LocalPackage,
    build: Option<&Build>,
    role: &str,
    current: Option<f64>,
) -> Option<f64> {
    let base = form.base_stats.as_ref()?;
    let iv = pokemon.ivs?;
    let ceiling = cpm(package, 40.0)?;
    let value = |m: f64| -> Option<f64> {
        if role == "ATTACKER" {
            let b = build?;
            return Some(cycle(
                base,
                iv.attack,
                form.types.as_ref()?,
                pair(package.combat.as_ref()?, &b.fast_move, &b.charged_move)?,
                m,
                cpm(package, 30.0)?,
            ));
        }
        let hp = ((f64::from(base.stamina) + f64::from(iv.hp)) * m)
            .floor()
            .max(10.0);
        let bulk = (f64::from(base.defense) + f64::from(iv.defense)) * m * hp;
        Some(if role == "GYM DEF" {
            bulk
        } else {
            bulk * (f64::from(base.attack) + f64::from(iv.attack)) * m
        })
    };
    let target = value(ceiling)? * 0.9;
    package
        .cp_multipliers
        .iter()
        .filter(|(l, _)| *l <= 40.0 && current.is_none_or(|from| *l >= from))
        .find(|(_, m)| value(*m).is_some_and(|v| v >= target))
        .map(|(l, _)| *l)
}

pub fn plan(
    pokemon: &Pokemon,
    details: &DetailsView,
    package: Option<&LocalPackage>,
    gym_model: Option<&GymModel>,
    mut resources: ResourceBudget,
    global_stardust: Option<u32>,
    protected: bool,
    goal: &str,
    role_model: &crate::roles::RoleModel,
    preferred_forms: &[String],
) -> PreparationPlan {
    if resources.values.stardust.is_none() {
        resources.values.stardust = global_stardust;
    }
    let mut current_level = (!details.needs_reconfirmation)
        .then_some(details.values.level)
        .flatten();
    let mut result=PreparationPlan{primary_path_id:None,planning_mode:None,conflicts:vec![],selected_path_id:None,state:"conditional".into(),summary:"Compare keeping this Pokémon as it is with evolution, moves and level checkpoints. Review the proposed uses before spending.".into(),current_level,paths:vec![],missing:vec![],limitations:vec![
        "Levels 30, 40 and 50 are comparison checkpoints, not optimal battle breakpoints or automatic investment advice. Stop when your selected role is served; compare lower-cost owned alternatives.".into(),
        "Costs are summed from imported half-level rows for a manually confirmed ordinary cost profile. Lucky, Shadow, Purified, special species overrides and event discounts require an in-game quote; unsupported profiles remain unknown.".into(),
        "Gym benefit uses one zero-energy neutral cycle against fixed Defense 215 at level 30. No incoming damage, motivation, shields, fainting or battle simulation. Other roles must be reviewed separately.".into(),
        "Planning allocations are shared resources you choose to reserve for this individual, not separate family/global balances. Plans do not reserve or deduct resources. Confirm real costs in game.".into(),
    ],resources};
    let Some(package) = package else {
        result.state = "needs_data".into();
        result
            .missing
            .push("Install local preparation and combat references.".into());
        return result;
    };
    let Some(form) = pokemon.reference.form.as_ref() else {
        result.state = "needs_data".into();
        result.missing.push("Confirm reference identity.".into());
        return result;
    };
    if matches!((pokemon.cp,current_level.and_then(|l|cpm(package,l)).and_then(|m|cp(pokemon,form,m))),(Some(observed),Some(projected)) if observed!=i64::from(projected))
    {
        current_level = None;
        result.current_level = None;
        result.missing.push("Observed CP conflicts with the confirmed level, IVs and imported base stats. Reconfirm current evidence before exact costs or gains.".into());
    }
    let role = match goal {
        "gym_defense" => "GYM DEF",
        "rocket" => "ROCKET",
        "great" => "PVP 1500",
        "ultra" => "PVP 2500",
        "master" => "PVP MASTER",
        _ => "ATTACKER",
    };
    let trainer_goal = matches!(goal, "rocket" | "great" | "ultra" | "master");
    let reference = package.preparation.as_ref();
    if reference.is_none() {
        result.missing.push("Import pokemon_evolutions and pokemon_powerup_requirements with current move pools for preparation data.".into());
    }
    if current_level.is_none() {
        result.missing.push("Optional current level with fresh manual evidence is needed for exact Power Up totals; CP alone is not a level.".into());
    }
    // The upstream global cost table cannot represent the known species-level
    // Eternatus upgrade override in PokeMiners' POKEMON_UPGRADE_SETTINGS data.
    // Withhold totals rather than embed that game's override cost dataset.
    let special_cost_override = pokemon
        .reference
        .species
        .as_ref()
        .is_some_and(|s| s.dex_number == 890);
    let normal = !special_cost_override
        && result.resources.values.normal_costs_confirmed
        && !details.needs_reconfirmation
        && details.values.shadow != Some(true)
        && details.values.purified != Some(true);
    if !normal {
        result.missing.push("Confirm an ordinary, non-Lucky/non-Shadow/non-Purified cost profile or use in-game quotes. No modifier is guessed.".into());
    }
    if special_cost_override {
        result.missing.push("This species has a known special Power Up cost override. The imported global table is insufficient; use an in-game quote.".into());
    }
    let catalog = match package.validate() {
        Ok(catalog) => catalog,
        Err(_) => {
            result.state = "needs_data".into();
            return result;
        }
    };
    let mut states = vec![(
        pokemon.clone(),
        Vec::<crate::preparation_reference::EvolutionBranch>::new(),
    )];
    let mut index = 0;
    while index < states.len() && states.len() < 16 {
        let (source, chain) = states[index].clone();
        index += 1;
        if chain.len() >= 3 {
            continue;
        }
        let Some(from) = source.reference.form.as_ref() else {
            continue;
        };
        for (species, target) in catalog.evolution_targets(&source.reference) {
            if states.len() >= 16 {
                break;
            }
            let Some(edge) = reference
                .and_then(|r| {
                    r.evolutions
                        .iter()
                        .find(|e| e.from_form_id == from.id && e.to_form_id == target.id)
                })
                .cloned()
            else {
                continue;
            };
            if chain.iter().any(|e| e.from_form_id == target.id) {
                continue;
            }
            let mut next = source.clone();
            next.display_name = species.name.clone();
            next.reference.species = Some(species);
            next.reference.form = Some(target);
            next.cp = None;
            let mut path = chain.clone();
            path.push(edge);
            states.push((next, path));
        }
    }
    if form.evolutions.is_none() {
        result.missing.push(
            "Evolution coverage for this form is unknown; no-evolution is not inferred.".into(),
        );
    }
    let current_gym = gym_model.map(|m| m.assess(pokemon, Some(details)));
    let current_build = current_gym.as_ref().and_then(|g| g.builds.first());
    if states.len() > 4 {
        let current = states.remove(0);
        states.sort_by(|a, b| {
            let value = |p: &Pokemon| {
                let Some(f) = p.reference.form.as_ref() else {
                    return 0.0;
                };
                let Some(base) = f.base_stats.as_ref() else {
                    return 0.0;
                };
                if goal == "gym_defense" {
                    return f64::from(base.defense) * f64::from(base.stamina);
                }
                if matches!(goal, "great" | "ultra") {
                    let limit = if goal == "great" { 1500 } else { 2500 };
                    return league_level(p, f, package, limit)
                        .and_then(|l| cpm(package, l))
                        .map_or(0.0, |m| {
                            f64::from(base.attack)
                                * f64::from(base.defense)
                                * f64::from(base.stamina)
                                * m.powi(3)
                        });
                }
                if trainer_goal {
                    return package
                        .trainer_combat
                        .as_ref()
                        .and_then(|r| {
                            r.pools
                                .iter()
                                .find(|pool| pool.form_id == f.id)
                                .map(|pool| {
                                    r.moves
                                        .iter()
                                        .filter(|m| pool.fast_move_ids.contains(&m.id))
                                        .map(|m| f64::from(m.energy_delta) / f64::from(m.turns))
                                        .fold(0.0, f64::max)
                                })
                        })
                        .unwrap_or(0.0);
                }
                gym_model
                    .map(|m| m.assess(p, None))
                    .and_then(|g| g.builds.first().map(|b| b.neutral_cycle_dps))
                    .unwrap_or(0.0)
            };
            let preferred = |p: &Pokemon| {
                p.reference
                    .form
                    .as_ref()
                    .is_some_and(|f| preferred_forms.contains(&f.id))
            };
            preferred(&b.0).cmp(&preferred(&a.0)).then_with(|| {
                value(&b.0)
                    .total_cmp(&value(&a.0))
                    .then_with(|| a.0.display_name.cmp(&b.0.display_name))
            })
        });
        states.truncate(3);
        states.insert(0, current);
        result.limitations.push("The bounded display selects up to three reachable future forms using the selected role's proxy (gym cycle, defensive bulk, capped stat product or Trainer fast-energy access). This is not a battle ranking; other evolution branches can serve other roles and remain unassessed here.".into());
    }
    result.paths.push(PreparationPath{id:"keep-current-state".into(),target_form_id:form.id.clone(),target_name:pokemon.display_name.clone(),target_level:current_level,role:"retain".into(),actions:vec![PreparationAction{kind:"use_now".into(),description:"Retain the current observed state without spending. Consider current use after reviewing the role assessments.".into(),requirements:vec![],acquisition:vec![]}],cost:ResourceCost{candy:Some(0),candy_xl:Some(0),stardust:Some(0),items:vec![]},benefit:"Preserves your resources and future evolution/PvP options. This is not a claim of battle readiness.".into(),blockers:vec![],affordability:"within_budget".into(),executable:true,future_roles:vec![],served_roles:vec![],projected_cp:current_level.and_then(|l|cpm(package,l)).and_then(|m|cp(pokemon,form,m)),improvement_percent:None,target_kind:Some("retain".into())});
    let current_score = package
        .combat
        .as_ref()
        .filter(|_| details.values.shadow != Some(true) && !details.needs_reconfirmation)
        .and_then(|combat| {
            let b = form.base_stats.as_ref()?;
            let types = form.types.as_ref()?;
            let level = current_level?;
            // Actual current set, not the best hypothetical set, is the baseline.
            let fast = details.values.fast_move.as_deref()?;
            let attack = pokemon.ivs?.attack;
            let multiplier = cpm(package, level)?;
            let defender = cpm(package, 30.0)?;
            [
                &details.values.charged_move,
                &details.values.second_charged_move,
            ]
            .iter()
            .filter_map(|charged| pair(combat, fast, charged.as_deref()?))
            .map(|moves| cycle(b, attack, types, moves, multiplier, defender))
            .max_by(f64::total_cmp)
        });
    for (target, chain) in states {
        let Some(target_form) = target.reference.form.as_ref() else {
            continue;
        };
        let gym = if details.values.shadow == Some(true) {
            None
        } else {
            gym_model.map(|model| model.assess(&target, None))
        };
        let target_build = if trainer_goal {
            trainer_build(package, target_form)
        } else {
            gym.as_ref().and_then(|g| g.builds.first()).map(|b| Build {
                attack_type: b.attack_type.clone(),
                fast_move: b.fast_move.clone(),
                charged_move: b.charged_move.clone(),
                reference_summary: b.reference_summary.clone(),
                strong_against: b.strong_against.clone(),
            })
        };
        let build = target_build.as_ref();
        let efficient = if details.values.shadow == Some(true) {
            None
        } else {
            model_checkpoint(pokemon, target_form, package, build, role, current_level)
        };
        let mut levels = vec![current_level];
        let mut checkpoints = match goal {
            "great" => league_level(pokemon, target_form, package, 1500)
                .into_iter()
                .collect(),
            "ultra" => league_level(pokemon, target_form, package, 2500)
                .into_iter()
                .collect(),
            _ => vec![30.0, 40.0, 50.0],
        };
        if !matches!(goal, "great" | "ultra" | "master") {
            if let Some(level) = efficient {
                checkpoints.insert(0, level);
            }
        }
        if let Some(intent) = result.resources.values.route_intent.as_ref().filter(|i| {
            i.dataset_id == pokemon.reference.dataset.dataset_id
                && i.target_form_id == target_form.id
        }) {
            if let Some(level) = intent.target_level {
                checkpoints.insert(0, level);
            }
        }
        for checkpoint in checkpoints {
            if current_level.is_none_or(|l| l < checkpoint) && !levels.contains(&Some(checkpoint)) {
                levels.push(Some(checkpoint));
            }
        }
        for level in levels {
            if result.paths.len() >= MAX_PATHS {
                break;
            }
            let is_power = level.is_some() && level != current_level;
            if is_power
                && chain.is_empty()
                && matches!((pokemon.cp,level.and_then(|l|cpm(package,l)).and_then(|m|cp(pokemon,target_form,m))),(Some(observed),Some(projected)) if i64::from(projected)<observed)
            {
                result.missing.push("A lower projected CP is not a reachable same-form Power Up target. Reconfirm the current level before choosing further investment.".into());
                continue;
            }
            let mut actions = Vec::new();
            let mut blockers = Vec::new();
            let mut cost = ResourceCost {
                candy: Some(0),
                candy_xl: Some(0),
                stardust: Some(0),
                items: vec![],
            };
            for edge in &chain {
                cost.candy = cost.candy.and_then(|v| v.checked_add(edge.candy));
                let requirements = edge
                    .requirements
                    .iter()
                    .map(|r| r.description.clone())
                    .collect::<Vec<_>>();
                if !edge.requirements_complete {
                    blockers.push("Verify all evolution prerequisites in game: source coverage may omit quests, events or form restrictions.".into());
                }
                if edge.free_if_traded {
                    blockers.push("Trade discount is conditional; costs show the non-traded Candy branch. Trade history is not inferred.".into());
                }
                cost.items.extend(
                    edge.requirements
                        .iter()
                        .filter(|r| r.kind == "item")
                        .map(|r| r.description.clone()),
                );
                if !normal {
                    cost.candy = None;
                    blockers.push("Evolution Candy total needs a matching ordinary cost profile or an in-game quote.".into());
                }
                actions.push(PreparationAction{kind:"evolve".into(),description:format!("Evolve {} → {}. The ordinary reference branch lists {} Candy; see the path total and conditions for the applicable estimate.",form_name(package,&edge.from_form_id),form_name(package,&edge.to_form_id),edge.candy),requirements,acquisition:vec!["Verify gender, buddy quests, item, location, time and event requirements in game before confirming the evolution.".into()]});
            }
            if is_power {
                let total = reference.and_then(|r| {
                    current_level.and_then(|from| power_up_cost(r, from, level.unwrap()))
                });
                if normal {
                    cost.candy = cost
                        .candy
                        .and_then(|v| total.and_then(|c| v.checked_add(c.0)));
                    cost.candy_xl = total.map(|c| c.1);
                    cost.stardust = total.map(|c| c.2);
                } else {
                    cost.candy = None;
                    cost.candy_xl = None;
                    cost.stardust = None;
                }
                actions.push(PreparationAction{kind:"power_up".into(),description:format!("Power Up from {} to level {} in half-level increments.",current_level.map_or_else(||"an unconfirmed level".into(),|l|l.to_string()),level.unwrap()),requirements:vec!["Fresh current level, matching in-game cost profile, sufficient shared resources and trainer cap.".into()],acquisition:vec![]});
                if total.is_none() {
                    blockers.push(
                        "Exact starting level or a complete imported cost interval is missing."
                            .into(),
                    );
                }
                if level.and_then(|l| cpm(package, l)).is_none() {
                    blockers.push("Target-level CPM is not verified by the active source. Costs can be shown, but level-dependent benefit is unresolved.".into());
                }
                if !normal {
                    blockers.push(
                        "Ordinary cost profile is unconfirmed or unsupported for this variant."
                            .into(),
                    );
                }
                if result
                    .resources
                    .values
                    .maximum_level
                    .is_none_or(|cap| level.unwrap() > cap)
                {
                    blockers
                        .push("Confirm that the in-game Power Up cap permits this target.".into());
                }
            }
            let matches = chain.is_empty()
                && !details.needs_reconfirmation
                && build.is_some_and(|b| {
                    details
                        .values
                        .fast_move
                        .as_ref()
                        .is_some_and(|m| m.eq_ignore_ascii_case(&b.fast_move))
                        && [
                            &details.values.charged_move,
                            &details.values.second_charged_move,
                        ]
                        .iter()
                        .any(|m| {
                            m.as_ref()
                                .is_some_and(|m| m.eq_ignore_ascii_case(&b.charged_move))
                        })
                });
            if let Some(build) = build.filter(|_| goal != "gym_defense") {
                if !matches {
                    let access = reference
                        .and_then(|r| r.move_access.iter().find(|a| a.form_id == target_form.id));
                    let methods = vec![
                        acquisition(&build.fast_move, true, access),
                        acquisition(&build.charged_move, false, access),
                    ];
                    cost.items.push("Move acquisition: TM/Elite TM attempts or event availability must be confirmed; count unknown.".into());
                    blockers.push("Confirm desired attacks and acquisition resources after evolution; proposed moves are not observations.".into());
                    actions.push(PreparationAction{kind:"change_moves".into(),description:format!("Prepare {} + {} for {} coverage.",build.fast_move,build.charged_move,build.attack_type),requirements:vec!["Do not assume normal TMs obtain legacy moves. Shadow Frustration removal and special attacks require separate in-game verification.".into()],acquisition:methods});
                }
            } else if goal != "gym_defense" {
                blockers.push(
                    "No supported damaging move pair for target; battle benefit is unknown.".into(),
                );
            }
            let mut benefit = build.map_or_else(
                || "Target battle benefit is unknown.".into(),
                |b| {
                    format!(
                        "{} attack potential against {} types; {}",
                        b.attack_type,
                        if b.strong_against.is_empty() {
                            "no established favorable".into()
                        } else {
                            b.strong_against.join(", ")
                        },
                        b.reference_summary
                    )
                },
            );
            if goal == "gym_defense" {
                benefit="Defense preparation: compare projected bulk and type resistances with other defenders. This does not model motivation decay or time to defeat. Defender move selection and placement eligibility need in-game review.".into();
                blockers.push("Gym placement eligibility and defender move timing require review; attack-cycle moves are not recommended for defense.".into());
            }
            if trainer_goal {
                benefit = build.map_or_else(
                    || {
                        "No imported Trainer-Battle kit supports a move preparation estimate."
                            .into()
                    },
                    |b| {
                        format!(
                            "{} coverage against {} types. {}",
                            b.attack_type,
                            b.strong_against.join(", "),
                            b.reference_summary
                        )
                    },
                );
                blockers.push("Opponent, shields, cup/lineup eligibility and current matchup performance require review; energy access alone is not battle strength.".into());
            }
            if matches!(goal, "great" | "ultra") {
                let limit = if goal == "great" { 1500 } else { 2500 };
                if current_level.is_some_and(|l| {
                    cp(pokemon, target_form, cpm(package, l).unwrap_or(0.0))
                        .is_some_and(|cp| cp > limit)
                }) {
                    blockers.push("This projected form at the confirmed current level exceeds the league CP limit; Power Up cannot lower its level.".into());
                }
            }
            let target_score = package
                .combat
                .as_ref()
                .filter(|_| role == "ATTACKER")
                .and_then(|combat| {
                    let build = build?;
                    Some(cycle(
                        target_form.base_stats.as_ref()?,
                        pokemon.ivs?.attack,
                        target_form.types.as_ref()?,
                        pair(combat, &build.fast_move, &build.charged_move)?,
                        cpm(package, level?)?,
                        cpm(package, 30.0)?,
                    ))
                });
            if is_power && role == "ATTACKER" && target_score.is_none() {
                blockers.push("The investment benefit needs supported target stats, IVs, moves and level mechanics; known costs alone do not establish a useful Power Up.".into());
            }
            if let Some(score) = target_score {
                benefit.push_str(&format!(
                    " Fixed-scenario projected cycle: {score:.2} damage/s."
                ));
                if let Some(now) = current_score {
                    benefit.push_str(&format!(" Current confirmed set at the same fixed defender: {now:.2}; change {:+.1}%.",(score/now-1.0)*100.0));
                } else {
                    benefit.push_str(
                        " Current set/level is unknown; no current-to-future gain is claimed.",
                    );
                }
            } else {
                benefit.push_str(" Level-dependent damage is unavailable without source CPM and current evidence.");
            }
            if let Some(projected) = level
                .and_then(|l| cpm(package, l))
                .and_then(|m| cp(pokemon, target_form, m))
            {
                benefit.push_str(&format!(" Projected CP {projected}; not an observed CP."));
            }
            let checkpoint =
                level == efficient && is_power && !matches!(goal, "great" | "ultra" | "master");
            if checkpoint {
                benefit.push_str(" This is the lowest supplied half level reaching 90% of this form's level-40 modeled cycle/bulk/stat product. It is a preparation policy, not a battle breakpoint. Evolution stages precede Power Up so resulting CP and attacks can be reviewed before further spending.");
            }
            if gym
                .as_ref()
                .and_then(|g| g.builds.first())
                .zip(current_build)
                .is_some_and(|(future, now)| {
                    future.neutral_cycle_dps <= now.neutral_cycle_dps && !chain.is_empty()
                })
            {
                benefit.push_str(" This evolution does not improve the best neutral equal-level gym cycle; retain other-role options before evolving.");
            }
            if protected && !actions.is_empty() {
                blockers.push("This individual is protected or has retention properties; review preservation before investment.".into());
            }
            let affordability = affordability(&cost, &result.resources);
            if affordability == "over_budget" {
                blockers.push("Exceeds your allocated planning resources. Wait or choose a cheaper alternative.".into());
            }
            let no_action = actions.is_empty();
            if no_action {
                actions.push(PreparationAction {
                    kind: "use_now".into(),
                    description:
                        "Keep the current observed state; no evolution, Power Up or move spending."
                            .into(),
                    requirements: vec![
                        "Select an opponent/type context and check current HP before battling."
                            .into(),
                    ],
                    acquisition: vec![],
                });
            }
            let identity=crate::reference_store::digest(&serde_json::to_vec(&serde_json::json!({"role":role,"chain":chain,"targetForm":target_form.id,"fromLevel":current_level,"toLevel":level,"actions":actions,"cost":cost,"source":package.source_sha256})).unwrap());
            result.paths.push(PreparationPath {
                id: format!("path-{}", &identity[..24]),
                target_form_id: target_form.id.clone(),
                target_name: target.display_name.clone(),
                target_level: level,
                role: role.into(),
                actions,
                cost,
                benefit,
                executable: blockers.is_empty() && (no_action || affordability == "within_budget"),
                affordability,
                blockers,
                future_roles: if chain.is_empty() || is_power {
                    vec![]
                } else {
                    role_model.assess(&target, None, &[])
                },
                served_roles: vec![role.into()],
                projected_cp: level
                    .and_then(|l| cpm(package, l))
                    .and_then(|m| cp(pokemon, target_form, m)),
                improvement_percent: target_score
                    .zip(current_score)
                    .map(|(future, now)| (future / now - 1.0) * 100.0),
                target_kind: Some(
                    if checkpoint {
                        "model_checkpoint"
                    } else if matches!(goal, "great" | "ultra") {
                        "league_cap"
                    } else if is_power {
                        "comparison_checkpoint"
                    } else {
                        "current_level"
                    }
                    .into(),
                ),
            });
        }
    }
    if result.paths.iter().any(|p| p.executable) {
        result.state = "alternatives_available".into();
    }
    result.selected_path_id = result
        .resources
        .values
        .selected_path_id
        .clone()
        .filter(|id| result.paths.iter().any(|path| path.id == *id));
    if result.resources.values.selected_path_id.is_some() && result.selected_path_id.is_none() {
        result.missing.push("Your previous path no longer exists under current evidence. Select a new path; progress is not carried forward.".into());
    }
    if let Some(reference) = reference {
        result.limitations.extend(reference.limitations.clone());
    }
    result
}
