//! User-imported preparation mechanics. Normal costs are conditional, never
//! proof that an individual can evolve or has the required resources.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const VERSION: &str = "pogoapi-preparation-1";
type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreparationReference {
    pub version: String,
    pub power_up_costs: Vec<PowerUpCost>,
    pub evolutions: Vec<EvolutionBranch>,
    pub move_access: Vec<MoveAccess>,
    pub limitations: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PowerUpCost {
    pub from_level: f64,
    pub to_level: f64,
    pub candy: u32,
    pub candy_xl: u32,
    pub stardust: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvolutionBranch {
    pub from_form_id: String,
    pub to_form_id: String,
    pub candy: u32,
    pub requirements: Vec<EvolutionRequirement>,
    pub requirements_complete: bool,
    pub free_if_traded: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvolutionRequirement {
    pub kind: String,
    pub description: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MoveAccess {
    pub form_id: String,
    pub fast_moves: Vec<String>,
    pub charged_moves: Vec<String>,
    pub elite_fast_moves: Vec<String>,
    pub elite_charged_moves: Vec<String>,
}
fn integer(value: &Value) -> Result<u32> {
    value
        .as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .filter(|n| *n <= 10_000_000)
        .ok_or("Invalid preparation cost or identifier")
}
fn level(value: &Value) -> Result<f64> {
    let n = value.as_f64().ok_or("Invalid preparation level")?;
    if !n.is_finite() || !(1.0..=50.0).contains(&n) || (n * 2.0).fract() != 0.0 {
        return Err("Invalid preparation level");
    }
    Ok(n)
}
fn text(value: &Value) -> Result<String> {
    let s = value.as_str().ok_or("Invalid preparation text")?;
    if !crate::reference::valid_text(s, 160) {
        return Err("Invalid preparation text");
    }
    Ok(s.into())
}
fn names(value: &Value) -> Result<Vec<String>> {
    if value.is_null() {
        return Ok(Vec::new());
    }
    let rows = value
        .as_array()
        .filter(|rows| rows.len() <= 100)
        .ok_or("Invalid preparation move list")?;
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for v in rows {
        let s = text(v)?;
        if !seen.insert(s.to_lowercase()) {
            return Err("Duplicate preparation move");
        }
        out.push(s);
    }
    Ok(out)
}
fn form(row: &Value) -> Result<Option<String>> {
    if row["form"].as_str() != Some("Normal") {
        return Ok(None);
    }
    let id = integer(&row["pokemon_id"])?;
    if id == 0 || id > 65535 {
        return Err("Invalid preparation species");
    }
    Ok(Some(format!("dex-{id}-normal")))
}
pub fn normalize(value: &Value) -> Result<Option<PreparationReference>> {
    let has_costs = value.get("pokemon_powerup_requirements").is_some();
    let has_evolutions = value.get("pokemon_evolutions").is_some();
    let has_moves = value.get("current_pokemon_moves").is_some();
    if !has_costs && !has_evolutions && !has_moves {
        return Ok(None);
    }
    let mut result=PreparationReference {version:VERSION.into(),power_up_costs:vec![],evolutions:vec![],move_access:vec![],limitations:vec![
        "Power-up costs are an imported ordinary global table. Lucky, Shadow, Purified and species-specific overrides are not resolved; verify the in-game preview before spending.".into(),
        "Evolution requirements may omit quests, availability and special conditions. Every imported branch requires checking the in-game evolution preview; omitted branches are unknown.".into(),
        "Normal TMs are random and their required count is unknown. Elite lists do not establish a currently active evolution event or event window.".into(),
    ]};
    if has_costs {
        let rows = value["pokemon_powerup_requirements"]
            .as_object()
            .filter(|rows| rows.len() <= 100)
            .ok_or("Invalid power-up requirements table")?;
        for (key, row) in rows {
            let from = level(&row["current_level"])?;
            let key_level = key
                .parse::<f64>()
                .map_err(|_| "Invalid power-up table key")?;
            if key_level != from {
                return Err("Power-up level key disagrees with row");
            }
            result.power_up_costs.push(PowerUpCost {
                from_level: from,
                to_level: level(&row["level_after_powering"])?,
                candy: integer(&row["candy_to_upgrade"])?,
                candy_xl: integer(&row["xl_candy_to_upgrade"])?,
                stardust: integer(&row["stardust_to_upgrade"])?,
            });
        }
        result
            .power_up_costs
            .sort_by(|a, b| a.from_level.total_cmp(&b.from_level));
    }
    if has_evolutions {
        let rows = value["pokemon_evolutions"]
            .as_array()
            .filter(|rows| rows.len() <= 10000)
            .ok_or("Invalid evolutions table")?;
        for row in rows {
            let Some(from) = form(row)? else { continue };
            let branches = row["evolutions"]
                .as_array()
                .filter(|v| v.len() <= 100)
                .ok_or("Invalid evolution branches")?;
            for branch in branches {
                let Some(to) = form(branch)? else { continue };
                let mut requirements = vec![];
                for (key, kind, label) in [
                    ("item_required", "item", "Evolution item"),
                    ("lure_required", "lure", "Nearby lure"),
                    ("gender_required", "gender", "Required gender"),
                ] {
                    if let Some(v) = branch.get(key) {
                        requirements.push(EvolutionRequirement {
                            kind: kind.into(),
                            description: format!("{label}: {}", text(v)?),
                        });
                    }
                }
                for (key, kind, description) in [
                    (
                        "only_evolves_in_daytime",
                        "daytime",
                        "Evolve during daytime",
                    ),
                    (
                        "only_evolves_in_nighttime",
                        "nighttime",
                        "Evolve during nighttime",
                    ),
                    ("must_be_buddy_to_evolve", "buddy", "Must be your buddy"),
                    (
                        "only_evolves_during_full_moon",
                        "full_moon",
                        "Requires a full moon",
                    ),
                    ("upside_down", "orientation", "Requires device upside down"),
                ] {
                    if let Some(v) = branch.get(key) {
                        let b = v.as_bool().ok_or("Invalid evolution condition")?;
                        if b {
                            requirements.push(EvolutionRequirement {
                                kind: kind.into(),
                                description: description.into(),
                            });
                        }
                    }
                }
                if let Some(v) = branch.get("buddy_distance_required") {
                    let km = v
                        .as_f64()
                        .filter(|n| n.is_finite() && *n >= 0.0 && *n <= 10000.0)
                        .ok_or("Invalid buddy distance")?;
                    requirements.push(EvolutionRequirement {
                        kind: "buddy_distance".into(),
                        description: format!("Walk {km} km with this buddy"),
                    });
                }
                if branch.get("priority").is_some() {
                    requirements.push(EvolutionRequirement{kind:"branch_selection".into(),description:"Branch priority can interact with other eligible evolutions; verify the displayed target".into()});
                }
                result.evolutions.push(EvolutionBranch {
                    from_form_id: from.clone(),
                    to_form_id: to,
                    candy: integer(&branch["candy_required"])?,
                    requirements,
                    requirements_complete: false,
                    free_if_traded: match branch.get("no_candy_cost_if_traded") {
                        Some(v) => v.as_bool().ok_or("Invalid trade evolution flag")?,
                        None => false,
                    },
                });
            }
        }
    }
    if has_moves {
        let rows = value["current_pokemon_moves"]
            .as_array()
            .filter(|v| v.len() <= 10000)
            .ok_or("Invalid move access table")?;
        for row in rows {
            let Some(form_id) = form(row)? else { continue };
            result.move_access.push(MoveAccess {
                form_id,
                fast_moves: names(&row["fast_moves"])?,
                charged_moves: names(&row["charged_moves"])?,
                elite_fast_moves: names(&row["elite_fast_moves"])?,
                elite_charged_moves: names(&row["elite_charged_moves"])?,
            });
        }
    }
    let ids = result
        .evolutions
        .iter()
        .flat_map(|e| [e.from_form_id.clone(), e.to_form_id.clone()])
        .chain(result.move_access.iter().map(|m| m.form_id.clone()))
        .collect();
    validate(&result, &ids)?;
    Ok(Some(result))
}
pub fn validate(reference: &PreparationReference, form_ids: &BTreeSet<String>) -> Result<()> {
    if reference.version != VERSION
        || reference.power_up_costs.len() > 100
        || reference.evolutions.len() > 10000
        || reference.move_access.len() > 10000
        || reference.limitations.len() > 20
    {
        return Err("Unsupported preparation reference");
    }
    for s in &reference.limitations {
        if !crate::reference::valid_text(s, 500) {
            return Err("Invalid preparation limitation");
        }
    }
    let mut levels = BTreeSet::new();
    for row in &reference.power_up_costs {
        level(&serde_json::json!(row.from_level))?;
        level(&serde_json::json!(row.to_level))?;
        if row.to_level != row.from_level + 0.5
            || !levels.insert((row.from_level * 2.0) as u16)
            || row.candy > 10_000_000
            || row.candy_xl > 10_000_000
            || row.stardust > 10_000_000
        {
            return Err("Invalid power-up interval");
        }
    }
    let mut edges = BTreeSet::new();
    for edge in &reference.evolutions {
        if !form_ids.contains(&edge.from_form_id)
            || !form_ids.contains(&edge.to_form_id)
            || edge.from_form_id == edge.to_form_id
            || edge.candy > 10_000_000
            || edge.requirements.len() > 30
            || !edges.insert((&edge.from_form_id, &edge.to_form_id))
        {
            return Err("Invalid preparation evolution link");
        }
        for r in &edge.requirements {
            if !crate::reference::valid_text(&r.kind, 60)
                || !crate::reference::valid_text(&r.description, 300)
            {
                return Err("Invalid preparation requirement");
            }
        }
    }
    let mut pools = BTreeSet::new();
    for row in &reference.move_access {
        if !form_ids.contains(&row.form_id) || !pools.insert(&row.form_id) {
            return Err("Invalid preparation move form");
        }
        for list in [
            &row.fast_moves,
            &row.charged_moves,
            &row.elite_fast_moves,
            &row.elite_charged_moves,
        ] {
            if list.len() > 100 {
                return Err("Too many preparation moves");
            }
            let mut names = BTreeSet::new();
            for s in list {
                if !crate::reference::valid_text(s, 160) || !names.insert(s.to_lowercase()) {
                    return Err("Invalid preparation move");
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn source() -> Value {
        json!({
            "pokemon_powerup_requirements": {
                "19.5": {"current_level":19.5,"level_after_powering":20,"candy_to_upgrade":2,"xl_candy_to_upgrade":0,"stardust_to_upgrade":2500},
                "40": {"current_level":40,"level_after_powering":40.5,"candy_to_upgrade":0,"xl_candy_to_upgrade":10,"stardust_to_upgrade":10000}
            },
            "pokemon_evolutions":[
                {"pokemon_id":60000,"form":"Normal","evolutions":[
                    {"pokemon_id":60001,"form":"Normal","candy_required":100,"item_required":"Sample Stone","gender_required":"Female","upside_down":true,"no_candy_cost_if_traded":true}
                ]}
            ],
            "current_pokemon_moves":[
                {"pokemon_id":60001,"form":"Normal","fast_moves":["Sample Spark"],"charged_moves":["Sample Burst"],"elite_fast_moves":[],"elite_charged_moves":["Sample Legacy"]}
            ]
        })
    }

    #[test]
    fn normalizes_half_level_costs_requirements_and_elite_access_without_asserting_completeness() {
        let result = normalize(&source()).unwrap().unwrap();
        assert_eq!(result.power_up_costs[0].from_level, 19.5);
        assert_eq!(result.power_up_costs[0].candy_xl, 0);
        assert_eq!(result.power_up_costs[1].candy, 0);
        assert_eq!(result.power_up_costs[1].candy_xl, 10);
        let edge = &result.evolutions[0];
        assert_eq!(edge.from_form_id, "dex-60000-normal");
        assert_eq!(edge.to_form_id, "dex-60001-normal");
        assert_eq!(edge.candy, 100);
        assert!(edge.free_if_traded);
        assert!(!edge.requirements_complete);
        assert_eq!(
            edge.requirements
                .iter()
                .map(|r| r.kind.as_str())
                .collect::<Vec<_>>(),
            vec!["item", "gender", "orientation"]
        );
        assert_eq!(
            result.move_access[0].elite_charged_moves,
            vec!["Sample Legacy"]
        );
        assert!(!result.move_access[0]
            .charged_moves
            .contains(&"Sample Legacy".into()));
        let roundtrip: PreparationReference =
            serde_json::from_slice(&serde_json::to_vec(&result).unwrap()).unwrap();
        assert_eq!(roundtrip, result);
    }

    #[test]
    fn supports_independently_absent_endpoints_and_old_packages() {
        assert!(normalize(&json!({"pokemon_stats":[]})).unwrap().is_none());
        let moves=normalize(&json!({"current_pokemon_moves":[{"pokemon_id":60000,"form":"Normal","fast_moves":[],"charged_moves":[]}]})).unwrap().unwrap();
        assert!(moves.power_up_costs.is_empty());
        assert!(moves.evolutions.is_empty());
        assert_eq!(moves.move_access.len(), 1);
        assert!(
            normalize(&json!({"pokemon_evolutions":[],"pokemon_powerup_requirements":{}}))
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn rejects_malformed_costs_missing_endpoints_and_unlinked_forms() {
        for bad in [json!(-1), json!(1.5), json!(10000001), json!("10")] {
            let mut value = source();
            value["pokemon_powerup_requirements"]["40"]["xl_candy_to_upgrade"] = bad;
            assert!(normalize(&value).is_err());
        }
        let mut value = source();
        value["pokemon_powerup_requirements"]["40"]["level_after_powering"] = json!(41);
        assert!(normalize(&value).is_err());
        let mut value = source();
        value["pokemon_powerup_requirements"]["40"]["current_level"] = json!(39.5);
        assert!(normalize(&value).is_err());
        let reference = normalize(&source()).unwrap().unwrap();
        assert!(validate(&reference, &BTreeSet::from(["dex-60000-normal".into()])).is_err());
    }

    #[test]
    fn never_maps_ambiguous_forms_or_guesses_missing_form() {
        let value = json!({"pokemon_evolutions":[{"pokemon_id":60000,"form":"Normal","evolutions":[{"pokemon_id":60001,"form":"Alternate","candy_required":25},{"pokemon_id":60002,"candy_required":25}]}],"current_pokemon_moves":[{"pokemon_id":60000,"fast_moves":[],"charged_moves":[]}]});
        let reference = normalize(&value).unwrap().unwrap();
        assert!(reference.evolutions.is_empty());
        assert!(reference.move_access.is_empty());
    }

    #[test]
    fn rejects_duplicates_and_rejects_malformed_conditions() {
        let mut value = source();
        let duplicate = value["pokemon_evolutions"][0].clone();
        value["pokemon_evolutions"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(normalize(&value).is_err());
        let mut value = source();
        value["current_pokemon_moves"][0]["fast_moves"] = json!(["Sample Spark", "sample spark"]);
        assert!(normalize(&value).is_err());
        let mut value = source();
        value["pokemon_evolutions"][0]["evolutions"][0]["upside_down"] = json!("yes");
        assert!(normalize(&value).is_err());
    }
}
