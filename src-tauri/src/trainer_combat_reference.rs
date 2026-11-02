//! Optional Trainer-Battle mechanics from user-selected PoGoAPI files.
//! https://pogoapi.net/documentation/ (pvp_fast_moves / pvp_charged_moves).
//! Turn counts are supplied explicitly; PvE milliseconds are never substituted.
use crate::combat_reference::{CombatPool, TYPES};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: &str = "pogoapi-trainer-1";
type Result<T> = std::result::Result<T, &'static str>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainerMove {
    pub id: String,
    pub name: String,
    pub move_type: String,
    pub kind: String,
    pub power: f64,
    pub energy_delta: i16,
    pub turns: u16,
    pub has_buffs: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainerCombatReference {
    pub version: String,
    pub moves: Vec<TrainerMove>,
    pub pools: Vec<CombatPool>,
}

fn valid_id(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|suffix| {
        suffix
            .parse::<u32>()
            .is_ok_and(|number| number > 0 && number.to_string() == suffix)
    })
}

impl TrainerCombatReference {
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION
            || self.moves.is_empty()
            || self.moves.len() > 2000
            || self.pools.len() > 10000
        {
            return Err("Unsupported or oversized Trainer-Battle reference");
        }
        let mut moves = BTreeMap::new();
        let mut names = BTreeSet::new();
        for movement in &self.moves {
            if !valid_id(&movement.id, "move-")
                || !crate::reference::valid_text(&movement.name, 100)
                || !TYPES.contains(&movement.move_type.as_str())
                || !movement.power.is_finite()
                || !(0.0..=1000.0).contains(&movement.power)
                || !(1..=100).contains(&movement.turns)
                || (movement.kind != "fast" && movement.kind != "charged")
                || movement.kind == "fast" && !(0..=100).contains(&movement.energy_delta)
                || movement.kind == "charged" && !(-100..=-1).contains(&movement.energy_delta)
                || moves.insert(movement.id.as_str(), movement).is_some()
                || !names.insert(movement.name.to_lowercase())
            {
                return Err("Invalid or ambiguous Trainer-Battle move");
            }
        }
        let mut forms = BTreeSet::new();
        let mut pairs = 0usize;
        for pool in &self.pools {
            if !pool
                .form_id
                .strip_suffix("-normal")
                .is_some_and(|id| valid_id(id, "dex-"))
                || !forms.insert(&pool.form_id)
                || pool.withheld_moves.len() > 200
                || pool
                    .withheld_moves
                    .iter()
                    .any(|name| !crate::reference::valid_text(name, 120))
                || pool.withheld_moves.iter().collect::<BTreeSet<_>>().len()
                    != pool.withheld_moves.len()
            {
                return Err("Invalid Trainer-Battle pool identity or withheld evidence");
            }
            pairs = pairs
                .checked_add(
                    pool.fast_move_ids
                        .len()
                        .saturating_mul(pool.charged_move_ids.len()),
                )
                .ok_or("Trainer-Battle pair bound exceeded")?;
            if pairs > 250000 {
                return Err("Trainer-Battle pair bound exceeded");
            }
            for (ids, kind) in [
                (&pool.fast_move_ids, "fast"),
                (&pool.charged_move_ids, "charged"),
            ] {
                let mut unique = BTreeSet::new();
                if ids.len() > 100 {
                    return Err("Oversized Trainer-Battle pool");
                }
                for id in ids {
                    if !unique.insert(id)
                        || !moves
                            .get(id.as_str())
                            .is_some_and(|movement| movement.kind == kind)
                    {
                        return Err("Duplicate, missing or wrong-kind Trainer-Battle pool move");
                    }
                }
            }
        }
        Ok(())
    }
}

fn parse_moves(value: &Value, kind: &str) -> Result<Vec<TrainerMove>> {
    let rows = value
        .as_array()
        .filter(|rows| !rows.is_empty() && rows.len() <= 2000)
        .ok_or("Expected bounded Trainer-Battle move array")?;
    rows.iter()
        .map(|row| {
            let id = row["move_id"]
                .as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .filter(|id| *id > 0)
                .ok_or("Invalid Trainer-Battle move ID")?;
            let name = row["name"]
                .as_str()
                .filter(|name| crate::reference::valid_text(name, 100))
                .ok_or("Invalid Trainer-Battle move name")?;
            let move_type = row["type"]
                .as_str()
                .ok_or("Missing Trainer-Battle move type")?
                .to_ascii_lowercase();
            let power = row["power"]
                .as_f64()
                .filter(|power| power.is_finite() && (0.0..=10000.0).contains(power))
                .ok_or("Invalid Trainer-Battle move power")?;
            let energy_delta = row["energy_delta"]
                .as_i64()
                .and_then(|value| i16::try_from(value).ok())
                .filter(|value| (-100..=100).contains(value))
                .ok_or("Invalid Trainer-Battle move energy")?;
            let turns = row["turn_duration"]
                .as_u64()
                .and_then(|value| u16::try_from(value).ok())
                .filter(|value| (1..=100).contains(value))
                .ok_or("Invalid Trainer-Battle move turn duration")?;
            if !TYPES.contains(&move_type.as_str()) || kind == "fast" && energy_delta < 0 {
                return Err("Unsupported Trainer-Battle move kind or type");
            }
            let has_buffs = if row["buffs"].is_null() {
                false
            } else {
                let buffs = row["buffs"]
                    .as_object()
                    .ok_or("Invalid Trainer-Battle buffs object")?;
                let chance = row["buffs"]["activation_chance"]
                    .as_f64()
                    .filter(|chance| chance.is_finite() && (0.0..=1.0).contains(chance))
                    .ok_or("Invalid Trainer-Battle buff chance")?;
                for (key, value) in buffs {
                    if key != "activation_chance"
                        && (![
                            "target_defense_stat_stage_change",
                            "attacker_attack_stat_stage_change",
                            "target_attack_stat_stage_change",
                            "attacker_defense_stat_stage_change",
                        ]
                        .contains(&key.as_str())
                            || !value
                                .as_i64()
                                .is_some_and(|stage| (-4..=4).contains(&stage)))
                    {
                        return Err("Unsupported Trainer-Battle buff mechanics");
                    }
                }
                chance > 0.0
            };
            Ok(TrainerMove {
                id: format!("move-{id}"),
                name: name.into(),
                move_type,
                kind: kind.into(),
                power,
                energy_delta,
                turns,
                has_buffs,
            })
        })
        .collect()
}

/// Optional old catalog imports remain usable. Once both move endpoints are
/// supplied, malformed mechanics reject the import instead of silently falling
/// back to raid data. Unsupported/ambiguous named options remain explicit.
pub fn normalize(provider: &str, value: &Value) -> Result<Option<TrainerCombatReference>> {
    if provider != "pogoapi"
        || [
            "pvp_fast_moves",
            "pvp_charged_moves",
            "current_pokemon_moves",
        ]
        .iter()
        .any(|key| value.get(key).is_none())
    {
        return Ok(None);
    }
    let mut moves = parse_moves(&value["pvp_fast_moves"], "fast")?;
    moves.extend(parse_moves(&value["pvp_charged_moves"], "charged")?);
    let mut ids = BTreeSet::new();
    let mut counts = BTreeMap::<String, usize>::new();
    for movement in &moves {
        if !ids.insert(movement.id.clone()) {
            return Err("Duplicate Trainer-Battle move ID");
        }
        *counts.entry(movement.name.to_lowercase()).or_default() += 1;
    }
    moves.retain(|movement| {
        counts[&movement.name.to_lowercase()] == 1
            && movement.power <= 1000.0
            && (movement.kind != "charged" || movement.energy_delta < 0)
    });
    moves.sort_by(|a, b| a.id.cmp(&b.id));
    let by_name: BTreeMap<_, _> = moves
        .iter()
        .map(|movement| (movement.name.to_lowercase(), movement))
        .collect();
    let raw_pools = value["current_pokemon_moves"]
        .as_array()
        .filter(|rows| rows.len() <= 10000)
        .ok_or("Expected bounded current move pools")?;
    let mut pools = vec![];
    for row in raw_pools
        .iter()
        .filter(|row| row["form"].as_str() == Some("Normal"))
    {
        let dex = row["pokemon_id"]
            .as_u64()
            .and_then(|id| u16::try_from(id).ok())
            .filter(|id| *id > 0)
            .ok_or("Invalid Trainer-Battle pool Dex number")?;
        let mut withheld = BTreeSet::new();
        let mut map_pool = |field: &str, kind: &str| -> Result<Vec<String>> {
            let names = row[field]
                .as_array()
                .filter(|names| names.len() <= 100)
                .ok_or("Expected bounded current move names")?;
            let mut unique = BTreeSet::new();
            let mut result = vec![];
            for name in names {
                let name = name
                    .as_str()
                    .filter(|name| crate::reference::valid_text(name, 100))
                    .ok_or("Invalid current Trainer-Battle move name")?;
                if !unique.insert(name.to_lowercase()) {
                    return Err("Duplicate current Trainer-Battle move name");
                }
                match by_name.get(&name.to_lowercase()) {
                    Some(movement) if movement.kind == kind => result.push(movement.id.clone()),
                    Some(_) => {
                        return Err(
                            "Current Trainer-Battle pool uses a known move of the wrong kind",
                        )
                    }
                    None => {
                        withheld.insert(format!("{kind}: {name}"));
                    }
                }
            }
            result.sort();
            Ok(result)
        };
        let fast_move_ids = map_pool("fast_moves", "fast")?;
        let charged_move_ids = map_pool("charged_moves", "charged")?;
        pools.push(CombatPool {
            form_id: format!("dex-{dex}-normal"),
            fast_move_ids,
            charged_move_ids,
            withheld_moves: withheld.into_iter().collect(),
        });
    }
    pools.sort_by(|a, b| a.form_id.cmp(&b.form_id));
    let result = TrainerCombatReference {
        version: VERSION.into(),
        moves,
        pools,
    };
    result.validate()?;
    Ok(Some(result))
}

#[cfg(test)]
fn fictional_input() -> Value {
    serde_json::json!({
        "pvp_fast_moves":[{"move_id":900001,"name":"Sample Tap","type":"Normal","power":4,"energy_delta":8,"turn_duration":2}],
        "pvp_charged_moves":[{"move_id":900002,"name":"Sample Slam","type":"Normal","power":80,"energy_delta":-40,"turn_duration":1}],
        "current_pokemon_moves":[{"pokemon_id":61001,"form":"Normal","fast_moves":["Sample Tap"],"charged_moves":["Sample Slam"]}]
    })
}

#[cfg(test)]
pub(crate) fn fictional_reference() -> TrainerCombatReference {
    normalize("pogoapi", &fictional_input()).unwrap().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn supplied_turns_energy_and_power_are_not_pve_values() {
        let reference = fictional_reference();
        let fast = reference
            .moves
            .iter()
            .find(|movement| movement.kind == "fast")
            .unwrap();
        let charged = reference
            .moves
            .iter()
            .find(|movement| movement.kind == "charged")
            .unwrap();
        assert_eq!(fast.turns, 2);
        assert_eq!(fast.energy_delta, 8);
        assert_eq!(charged.power, 80.0);
        assert_eq!(charged.energy_delta, -40);
        assert!(!charged.has_buffs);
    }
    #[test]
    fn malformed_persistence_rejects_zero_turns_wrong_kind_and_missing_links() {
        let mut reference = fictional_reference();
        reference.moves[0].turns = 0;
        assert!(reference.validate().is_err());
        let mut reference = fictional_reference();
        reference.pools[0].fast_move_ids = reference.pools[0].charged_move_ids.clone();
        assert!(reference.validate().is_err());
        let mut reference = fictional_reference();
        reference.pools[0].charged_move_ids.push("move-123".into());
        assert!(reference.validate().is_err());
    }
    #[test]
    fn missing_supplement_is_unknown_not_fabricated() {
        assert!(
            normalize("pogoapi", &json!({"fast_moves":[],"charged_moves":[]}))
                .unwrap()
                .is_none()
        );
        assert!(normalize(
            "pogoapi",
            &json!({"pvp_fast_moves":[],"pvp_charged_moves":[],"current_pokemon_moves":[]})
        )
        .is_err());
    }
    #[test]
    fn unsupported_name_is_withheld_without_guessing_a_move_id() {
        let mut input = fictional_input();
        input["current_pokemon_moves"][0]["charged_moves"] =
            json!(["Sample Slam", "Unmapped Attack"]);
        let reference = normalize("pogoapi", &input).unwrap().unwrap();
        assert_eq!(
            reference.pools[0].withheld_moves,
            vec!["charged: Unmapped Attack"]
        );
        assert_eq!(reference.pools[0].charged_move_ids, vec!["move-900002"]);
        input["pvp_charged_moves"][0]["energy_delta"] = json!(4);
        let reference = normalize("pogoapi", &input).unwrap().unwrap();
        assert!(reference.pools[0].charged_move_ids.is_empty());
        assert!(reference.pools[0]
            .withheld_moves
            .contains(&"charged: Sample Slam".into()));
    }

    #[test]
    fn ambiguity_and_wrong_kind_never_select_a_named_move_by_accident() {
        let mut input = fictional_input();
        let mut duplicate = input["pvp_charged_moves"][0].clone();
        duplicate["move_id"] = json!(900003);
        input["pvp_charged_moves"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        let reference = normalize("pogoapi", &input).unwrap().unwrap();
        assert!(reference.pools[0].charged_move_ids.is_empty());
        assert_eq!(
            reference.pools[0].withheld_moves,
            vec!["charged: Sample Slam"]
        );
        let mut input = fictional_input();
        input["current_pokemon_moves"][0]["fast_moves"] = json!(["Sample Slam"]);
        assert!(normalize("pogoapi", &input).is_err());
    }

    #[test]
    fn buffs_are_preserved_as_an_explicit_model_limit_and_invalid_data_rejects() {
        let mut input = fictional_input();
        input["pvp_charged_moves"][0]["buffs"] =
            json!({"activation_chance":1.0,"attacker_attack_stat_stage_change":2});
        assert!(normalize("pogoapi", &input)
            .unwrap()
            .unwrap()
            .moves
            .iter()
            .any(|movement| movement.kind == "charged" && movement.has_buffs));
        input["pvp_charged_moves"][0]["buffs"]["activation_chance"] = json!(1.1);
        assert!(normalize("pogoapi", &input).is_err());
    }
}
