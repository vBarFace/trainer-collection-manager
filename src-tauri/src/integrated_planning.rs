//! Joint preparation proposals. Role suggestions remain distinct from actual
//! readiness; shared game actions do not prove every battle role is usable.
use crate::{
    decision::IndividualDecision,
    gym::GymModel,
    individual_details::DetailsView,
    inventory::Pokemon,
    preparation::{self, PreparationPath, PreparationPlan},
    preparation_resources::ResourceBudget,
    reference::Form,
    reference_store::LocalPackage,
    roles::RoleModel,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: &str = "integrated-planning-1";
const MAX_DISPLAY_PATHS: usize = 16;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanConflict {
    pub path_id: String,
    pub role: String,
    pub explanation: String,
    pub severity: String,
}

/// Build supported alternatives without using a chosen global role as a filter.
/// The bounded final display is selected after the multi-role policy evaluates
/// current/future value, so a weak first role cannot crowd out a strong future one.
pub fn candidates(
    pokemon: &Pokemon,
    details: &DetailsView,
    package: Option<&LocalPackage>,
    gym: Option<&GymModel>,
    resources: ResourceBudget,
    stardust: Option<u32>,
    protected: bool,
    roles: &RoleModel,
    decision: &IndividualDecision,
) -> PreparationPlan {
    let preferred = |role: &str| {
        let mut forms = decision
            .suggestions
            .iter()
            .filter(|s| qualified(&s.state) && s.tag == role)
            .filter_map(|s| s.target_form_id.clone())
            .collect::<Vec<_>>();
        if let Some(intent) = resources.values.route_intent.as_ref().filter(|i| {
            i.dataset_id == pokemon.reference.dataset.dataset_id
                && i.roles.iter().any(|r| r == role)
        }) {
            forms.push(intent.target_form_id.clone());
        }
        forms
    };
    let mut result = preparation::plan(
        pokemon,
        details,
        package,
        gym,
        resources.clone(),
        stardust,
        protected,
        "gym_attack",
        roles,
        &preferred("ATTACKER"),
    );
    result.planning_mode = Some("automatic_multi_role".into());
    if package.is_none() || result.paths.is_empty() {
        return result;
    }
    let mut seen: BTreeSet<String> = result.paths.iter().map(|p| p.id.clone()).collect();
    for goal in ["gym_defense", "rocket", "great", "ultra", "master"] {
        let role = match goal {
            "gym_defense" => "GYM DEF",
            "rocket" => "ROCKET",
            "great" => "PVP 1500",
            "ultra" => "PVP 2500",
            _ => "PVP MASTER",
        };
        let other = preparation::plan(
            pokemon,
            details,
            package,
            gym,
            resources.clone(),
            stardust,
            protected,
            goal,
            roles,
            &preferred(role),
        );
        for path in other.paths {
            if seen.insert(path.id.clone()) {
                result.paths.push(path);
            }
        }
        result.missing.extend(other.missing);
    }
    result.missing.sort();
    result.missing.dedup();
    result.summary="The app evaluates all supported uses and proposes the next route automatically. Compatible actions can serve several uses; review role evidence and CP/move conflicts before an irreversible step. Applied tags and game facts remain your confirmations.".into();
    result.limitations.push("The automatic proposal uses qualified role evidence, preservation, affordability and a declared model checkpoint. It is not a universal quality score or competitive battle optimizer. PvP stat/move potential remains under review until competitive evidence exists.".into());
    result
}

fn qualified(state: &str) -> bool {
    matches!(state, "supported_current" | "preparation_potential")
}
fn qualifies(path: &PreparationPath, decision: &IndividualDecision) -> bool {
    decision.suggestions.iter().any(|s| {
        qualified(&s.state)
            && s.target_form_id.as_ref() == Some(&path.target_form_id)
            && path.served_roles.contains(&s.tag)
    })
}
fn current_use(path: &PreparationPath, decision: &IndividualDecision) -> bool {
    path.actions.iter().all(|a| a.kind == "use_now")
        && decision.suggestions.iter().any(|s| {
            s.state == "supported_current"
                && s.target_form_id.as_ref() == Some(&path.target_form_id)
                && path.served_roles.contains(&s.tag)
        })
}
fn contains_irreversible(path: &PreparationPath) -> bool {
    path.actions
        .iter()
        .any(|a| matches!(a.kind.as_str(), "power_up" | "evolve" | "change_moves"))
}
fn role_priority(path: &PreparationPath) -> u8 {
    if path.role == "ATTACKER" || path.served_roles.iter().any(|r| r == "ATTACKER") {
        3
    } else if path.role == "GYM DEF" || path.served_roles.iter().any(|r| r == "GYM DEF") {
        2
    } else if path.role == "ROCKET" || path.served_roles.iter().any(|r| r == "ROCKET") {
        1
    } else {
        0
    }
}
fn target_rank(path: &PreparationPath) -> u8 {
    match path.target_kind.as_deref() {
        Some("model_checkpoint") => 3,
        Some("current_level") => 2,
        Some("league_cap") => 1,
        _ => 0,
    }
}

/// Preserve explicit route choices; otherwise recommend qualified low-impact
/// routes before costly comparison targets. No automatic proposal is saved as a
/// user choice. This method runs before bounded roadmap serialization.
pub fn finalize(
    plan: &mut PreparationPlan,
    decision: &IndividualDecision,
    pokemon: &Pokemon,
    details: &DetailsView,
    package: Option<&LocalPackage>,
    protected_leagues: &[u32],
) {
    if plan.paths.is_empty() {
        return;
    }
    // Merge identical target/actions/costs. Keep manual IDs stable, and retain
    // each role's conditional evidence in the separate individual assessment.
    let selected = plan.resources.values.selected_path_id.clone();
    let mut groups = BTreeMap::<String, Vec<PreparationPath>>::new();
    for mut path in std::mem::take(&mut plan.paths) {
        path.served_roles.retain(|role| {
            decision.suggestions.iter().any(|s| {
                qualified(&s.state)
                    && s.tag == *role
                    && s.target_form_id.as_ref() == Some(&path.target_form_id)
            })
        });
        let key=crate::reference_store::digest(&serde_json::to_vec(&serde_json::json!({"target":path.target_form_id,"level":path.target_level,"actions":path.actions,"cost":path.cost,"unqualifiedComparisonRole":if path.served_roles.is_empty(){Some(&path.role)}else{None}})).unwrap());
        groups.entry(key).or_default().push(path);
    }
    let mut paths = Vec::new();
    for (_, mut group) in groups {
        group.sort_by(|a, b| {
            (selected.as_ref() == Some(&b.id))
                .cmp(&(selected.as_ref() == Some(&a.id)))
                .then_with(|| role_priority(b).cmp(&role_priority(a)))
                .then_with(|| a.id.cmp(&b.id))
        });
        let mut representative = group.remove(0);
        for other in group {
            representative.executable &= other.executable;
            representative.blockers.extend(other.blockers);
            representative.served_roles.extend(other.served_roles);
        }
        representative.blockers.sort();
        representative.blockers.dedup();
        representative.served_roles.sort();
        representative.served_roles.dedup();
        paths.push(representative);
    }
    plan.selected_path_id = selected.filter(|id| paths.iter().any(|p| p.id == *id));
    if plan.selected_path_id.is_none() {
        if let Some(intent) = plan
            .resources
            .values
            .route_intent
            .as_ref()
            .filter(|i| i.dataset_id == pokemon.reference.dataset.dataset_id)
        {
            let distance = |path: &PreparationPath| match (intent.target_level, path.target_level) {
                (Some(a), Some(b)) => (a - b).abs(),
                (None, None) => 0.0,
                (None, Some(_)) => {
                    if path.actions.iter().any(|a| a.kind == "power_up") {
                        2.0
                    } else {
                        1.0
                    }
                }
                _ => f64::INFINITY,
            };
            let mut matches = paths
                .iter()
                .filter(|p| {
                    p.target_form_id == intent.target_form_id
                        && (intent.roles.contains(&p.role)
                            || p.served_roles.iter().any(|r| intent.roles.contains(r)))
                })
                .collect::<Vec<_>>();
            matches.sort_by(|a, b| {
                distance(a)
                    .total_cmp(&distance(b))
                    .then_with(|| b.served_roles.len().cmp(&a.served_roles.len()))
                    .then_with(|| a.actions.len().cmp(&b.actions.len()))
                    .then_with(|| a.id.cmp(&b.id))
            });
            plan.selected_path_id = matches.first().map(|p| p.id.clone());
        }
    }
    // Preserve a real observed cap-bound option. Unknown future level cannot
    // certify that an evolution preserves eligibility, and is shown for review.
    let forms: BTreeMap<String, Form> = package
        .and_then(|p| p.catalog["forms"].as_array())
        .map_or_else(BTreeMap::new, |rows| {
            rows.iter()
                .filter_map(|r| {
                    serde_json::from_value::<Form>(r.clone())
                        .ok()
                        .map(|f| (f.id.clone(), f))
                })
                .collect()
        });
    let mut conflicts = Vec::new();
    for path in &mut paths {
        if !contains_irreversible(path) {
            continue;
        }
        for (tag, limit) in [("PVP 1500", 1500), ("PVP 2500", 2500)] {
            let evolving = path.actions.iter().any(|a| a.kind == "evolve");
            let current_option = decision.suggestions.iter().any(|s| {
                s.tag == tag
                    && s.target_form_id == pokemon.reference.form.as_ref().map(|f| f.id.clone())
            });
            let has_option =
                decision.suggestions.iter().any(|s| {
                    s.tag == tag && s.target_form_id.as_ref() == Some(&path.target_form_id)
                }) || (evolving && current_option);
            if !has_option && !protected_leagues.contains(&limit) {
                continue;
            }
            let current_projected = package
                .and_then(|p| plan.current_level.and_then(|l| preparation::cpm(p, l)))
                .and_then(|m| {
                    forms
                        .get(&path.target_form_id)
                        .and_then(|f| preparation::cp(pokemon, f, m))
                });
            let old_over = current_projected.is_some_and(|cp| cp > limit);
            let already_over = pokemon
                .reference
                .form
                .as_ref()
                .is_some_and(|f| f.id == path.target_form_id)
                && pokemon.cp.is_some_and(|cp| cp > i64::from(limit));
            if already_over {
                continue;
            }
            let exceeds = path.projected_cp.is_some_and(|cp| cp > limit);
            let current_form_option =
                evolving && current_option && pokemon.cp.is_none_or(|cp| cp <= i64::from(limit));
            let unknown = path.projected_cp.is_none()
                && path
                    .actions
                    .iter()
                    .any(|a| a.kind == "evolve" || a.kind == "power_up");
            if exceeds || unknown || current_form_option {
                let severity = if protected_leagues.contains(&limit) && (exceeds || unknown) {
                    "blocked"
                } else {
                    "review"
                };
                let explanation = if current_form_option && !exceeds && !unknown {
                    format!("Evolution replaces the current form and its unresolved {limit}-CP option, even if the evolved form remains below that cap. Compare current and future roles before confirming; competitive suitability has not been established.")
                } else if exceeds {
                    format!("This route projects {} CP for {} and would close or bypass the {limit}-CP option. {} Review the lower-CP alternatives or preserve another individual before evolving/Power Up.",path.projected_cp.unwrap(),path.target_name,if old_over{"At the current observed level this future form already exceeds that cap; it cannot be powered down."}else{"That league's competitive usefulness remains unresolved; stat potential alone is not a battle recommendation."})
                } else {
                    format!("The target CP is unknown, so this route cannot confirm preservation of the {limit}-CP option. Confirm level/IV/reference evidence before an irreversible step.")
                };
                path.executable = false;
                path.blockers.push(explanation.clone());
                conflicts.push(PlanConflict {
                    path_id: path.id.clone(),
                    role: tag.into(),
                    explanation,
                    severity: severity.into(),
                });
            }
        }
    }
    // Preserve observed special/legacy move value before proposing a reroll.
    // This is a safeguard, not a fabricated rarity or battle-strength claim.
    let special_moves = package
        .and_then(|p| p.preparation.as_ref())
        .and_then(|r| {
            pokemon
                .reference
                .form
                .as_ref()
                .and_then(|f| r.move_access.iter().find(|a| a.form_id == f.id))
        })
        .map_or_else(Vec::new, |access| {
            [
                (
                    &details.values.fast_move,
                    &access.fast_moves,
                    &access.elite_fast_moves,
                ),
                (
                    &details.values.charged_move,
                    &access.charged_moves,
                    &access.elite_charged_moves,
                ),
                (
                    &details.values.second_charged_move,
                    &access.charged_moves,
                    &access.elite_charged_moves,
                ),
            ]
            .iter()
            .filter_map(|(observed, ordinary, elite)| {
                observed
                    .as_ref()
                    .filter(|name| {
                        elite.iter().any(|m| m.eq_ignore_ascii_case(name))
                            || (!ordinary.is_empty()
                                && !ordinary.iter().any(|m| m.eq_ignore_ascii_case(name)))
                    })
                    .cloned()
            })
            .collect::<Vec<_>>()
        });
    if !special_moves.is_empty() {
        for path in &paths {
            if path
                .actions
                .iter()
                .any(|a| a.kind == "evolve" || a.kind == "change_moves")
            {
                conflicts.push(PlanConflict{path_id:path.id.clone(),role:"Move preservation".into(),severity:"review".into(),explanation:format!("Confirmed or previously observed attacks ({}) include Elite/special options or names absent from the ordinary pool. Preserve their acquisition value; reconfirm before evolving or replacing attacks. A normal TM may not restore them.",special_moves.join(", "))});
            }
        }
    }
    for path in paths
        .iter()
        .filter(|p| p.actions.iter().any(|a| a.kind == "evolve"))
    {
        let mut alternatives = BTreeMap::<String, BTreeSet<String>>::new();
        for use_ in decision.suggestions.iter().filter(|s| {
            qualified(&s.state)
                && s.target_form_id
                    .as_ref()
                    .is_some_and(|id| *id != path.target_form_id)
        }) {
            if let Some(name) = &use_.target_name {
                alternatives
                    .entry(name.clone())
                    .or_default()
                    .insert(use_.tag.clone());
            }
        }
        if !alternatives.is_empty() {
            let names = alternatives
                .into_iter()
                .map(|(name, roles)| {
                    format!(
                        "{name} ({})",
                        roles.into_iter().collect::<Vec<_>>().join(" / ")
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            conflicts.push(PlanConflict{path_id:path.id.clone(),role:"Evolution alternatives".into(),severity:"review".into(),explanation:format!("This route ends as {}. Other supported current/future uses end in different forms: {}. The same individual cannot retain every form at once; review each evolution stage or keep another useful individual for the other route.",path.target_name,names)});
        }
    }
    // Lexicographic policy keeps domain criteria separate; no summed universal
    // quality grade is produced. Exact ties use stable route IDs.
    paths.sort_by(|a, b| {
        current_use(b, decision)
            .cmp(&current_use(a, decision))
            .then_with(|| qualifies(b, decision).cmp(&qualifies(a, decision)))
            .then_with(|| {
                (b.affordability != "over_budget").cmp(&(a.affordability != "over_budget"))
            })
            .then_with(|| role_priority(b).cmp(&role_priority(a)))
            .then_with(|| target_rank(b).cmp(&target_rank(a)))
            .then_with(|| b.served_roles.len().cmp(&a.served_roles.len()))
            .then_with(|| a.actions.len().cmp(&b.actions.len()))
            .then_with(|| {
                a.cost
                    .stardust
                    .unwrap_or(u32::MAX)
                    .cmp(&b.cost.stardust.unwrap_or(u32::MAX))
            })
            .then_with(|| a.id.cmp(&b.id))
    });
    let blocked = |id: &str| {
        conflicts
            .iter()
            .any(|c| c.path_id == id && c.severity == "blocked")
    };
    let preservation = decision
        .suggestions
        .iter()
        .any(|s| s.tag == "SAVE/SPECIAL" && s.state == "collection_value");
    let primary = paths
        .iter()
        .find(|p| {
            qualifies(p, decision)
                && p.affordability != "over_budget"
                && !blocked(&p.id)
                && !(preservation && contains_irreversible(p))
                && !conflicts
                    .iter()
                    .any(|c| c.path_id == p.id && c.role == "Move preservation")
                && !(p.improvement_percent.is_some_and(|gain| gain < -1e-9)
                    && pokemon
                        .reference
                        .form
                        .as_ref()
                        .is_some_and(|f| f.id == p.target_form_id))
        })
        .map(|p| p.id.clone())
        .or_else(|| {
            paths
                .iter()
                .find(|p| p.id == "keep-current-state")
                .map(|p| p.id.clone())
        });
    if let Some(route) = plan
        .selected_path_id
        .as_ref()
        .or(primary.as_ref())
        .and_then(|id| paths.iter().find(|p| p.id == *id))
    {
        for use_ in decision.suggestions.iter().filter(|s| {
            qualified(&s.state)
                && s.target_form_id.as_ref() == Some(&route.target_form_id)
                && !route.served_roles.contains(&s.tag)
        }) {
            if paths.iter().any(|p| {
                p.target_form_id == route.target_form_id && p.served_roles.contains(&use_.tag)
            }) {
                conflicts.push(PlanConflict{path_id:route.id.clone(),role:use_.tag.clone(),severity:"review".into(),explanation:format!("The proposed route does not establish the separate {} preparation. Its move/level requirements differ or remain unverified. Compare the other route before replacing attacks or assuming both uses are ready; retaining another useful individual is valid.",use_.tag)});
            }
        }
    }
    for path in &mut paths {
        for conflict in conflicts.iter().filter(|c| c.path_id == path.id) {
            path.executable = false;
            if !path.blockers.contains(&conflict.explanation) {
                path.blockers.push(conflict.explanation.clone());
            }
        }
    }
    // Pin primary/manual/retain and at least one option for each domain, then
    // fill the remaining display slots deterministically.
    let mut pinned = BTreeSet::new();
    for id in [&plan.selected_path_id, &primary] {
        if let Some(id) = id {
            pinned.insert(id.clone());
        }
    }
    if paths.iter().any(|p| p.id == "keep-current-state") {
        pinned.insert("keep-current-state".into());
    }
    for role in ["ATTACKER", "GYM DEF", "ROCKET", "PVP 1500", "PVP 2500"] {
        if let Some(path) = paths
            .iter()
            .find(|p| p.role == role || p.served_roles.iter().any(|r| r == role))
        {
            pinned.insert(path.id.clone());
        }
    }
    let mut retained = Vec::new();
    for path in &paths {
        if pinned.contains(&path.id) {
            retained.push(path.clone());
        }
    }
    for path in paths {
        if retained.len() >= MAX_DISPLAY_PATHS {
            break;
        }
        if !pinned.contains(&path.id) {
            retained.push(path);
        }
    }
    for path in &mut retained {
        // Calibrated current/future role evidence is stored once in decision.
        // Repeating four legacy assessments per form can overflow the bounded
        // roadmap for branching species and suggest weaker, unqualified roles.
        path.future_roles.clear();
    }
    conflicts.retain(|c| retained.iter().any(|p| p.id == c.path_id));
    conflicts.sort_by(|a, b| a.path_id.cmp(&b.path_id).then_with(|| a.role.cmp(&b.role)));
    plan.primary_path_id = primary;
    plan.paths = retained;
    plan.conflicts = conflicts;
    for use_ in decision.suggestions.iter().filter(|s| qualified(&s.state)) {
        if !plan.paths.iter().any(|p| {
            use_.target_form_id.as_ref() == Some(&p.target_form_id)
                && p.served_roles.contains(&use_.tag)
        }) {
            plan.missing.push(format!("{} potential for {} needs a separately verified route beyond the bounded alternatives shown here; no preparation step is invented.",use_.tag,use_.target_name.as_deref().unwrap_or("the future form")));
        }
    }
    plan.missing
        .retain(|m| !m.starts_with("Your previous path no longer exists"));
    if plan.resources.values.selected_path_id.is_some() && plan.selected_path_id.is_none() {
        plan.missing.push("Your previous route no longer matches current evidence. The app proposes a fresh route while preserving the former roadmap as history.".into());
    }
    if plan.selected_path_id.is_some() {
        plan.summary="Your chosen target remains the active route where it is still reachable. Steps, costs and conflicts are revalidated against the current individual; no old action or observation is carried forward as confirmed. The app's automatic proposal remains available separately.".into();
    } else if plan.primary_path_id.as_deref() == Some("keep-current-state") {
        plan.summary="Retain this individual for now. No sufficiently supported preparation route justifies automatic investment under the available evidence. Review the suggested uses, missing facts and optional alternatives; ownership of another specimen alone is not a reason to transfer.".into();
    }
}
