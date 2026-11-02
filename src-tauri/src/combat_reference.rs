//! Optional, versioned PvE reference fields from user-selected PoGoAPI files.
//! No network, bundled game records, inferred moves or cross-provider guessing.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: &str = "pogoapi-pve-1";
pub const TYPES: [&str; 18] = [
    "normal", "fire", "water", "electric", "grass", "ice", "fighting", "poison", "ground",
    "flying", "psychic", "bug", "rock", "ghost", "dragon", "dark", "steel", "fairy",
];
const KEYS: [&str; 4] = [
    "fast_moves",
    "charged_moves",
    "current_pokemon_moves",
    "type_effectiveness",
];
type Result<T> = std::result::Result<T, &'static str>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatMove {
    pub id: String,
    pub name: String,
    pub move_type: String,
    pub kind: String,
    pub power: f64,
    pub energy_delta: i16,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatPool {
    pub form_id: String,
    pub fast_move_ids: Vec<String>,
    pub charged_move_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub withheld_moves: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatReference {
    pub version: String,
    pub moves: Vec<CombatMove>,
    pub pools: Vec<CombatPool>,
    /// Attacking move type -> defending type -> multiplier, without a supplied
    /// default. Dual-type effectiveness is computed by the consuming ruleset.
    pub effectiveness: BTreeMap<String, BTreeMap<String, f64>>,
}

fn canonical_type(value: &str) -> Result<String> {
    let normalized = value.to_ascii_lowercase();
    TYPES
        .contains(&normalized.as_str())
        .then_some(normalized)
        .ok_or("Unknown combat reference type")
}

fn valid_name(value: &str) -> bool {
    crate::reference::valid_text(value, 100) && value.trim() == value
}

fn name_key(value: &str) -> String {
    value.to_lowercase()
}

fn id_number(value: &str, prefix: &str) -> Option<u32> {
    let raw = value.strip_prefix(prefix)?;
    let number = raw.parse::<u32>().ok()?;
    (number > 0 && raw == number.to_string()).then_some(number)
}

fn valid_form_id(value: &str) -> bool {
    value
        .strip_suffix("-normal")
        .is_some_and(|id| id_number(id, "dex-").is_some_and(|dex| dex <= u32::from(u16::MAX)))
}

impl CombatReference {
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION
            || self.moves.is_empty()
            || self.moves.len() > 2_000
            || self.pools.is_empty()
            || self.pools.len() > 10_000
            || self.effectiveness.len() != TYPES.len()
        {
            return Err("Unsupported or oversized combat reference");
        }
        for (attacking, row) in &self.effectiveness {
            if !TYPES.contains(&attacking.as_str()) || row.len() != TYPES.len() {
                return Err("Combat type chart must cover all canonical types");
            }
            for (defending, multiplier) in row {
                if !TYPES.contains(&defending.as_str())
                    || !multiplier.is_finite()
                    || !(0.01..=4.0).contains(multiplier)
                {
                    return Err("Invalid combat type effectiveness multiplier");
                }
            }
        }
        let mut by_id = BTreeMap::new();
        let mut names = BTreeSet::new();
        for movement in &self.moves {
            if id_number(&movement.id, "move-").is_none()
                || !valid_name(&movement.name)
                || !TYPES.contains(&movement.move_type.as_str())
                || !matches!(movement.kind.as_str(), "fast" | "charged")
                || !movement.power.is_finite()
                || !(0.0..=1_000.0).contains(&movement.power)
                || !(1..=60_000).contains(&movement.duration_ms)
                || movement.kind == "fast" && !(0..=100).contains(&movement.energy_delta)
                || movement.kind == "charged" && !(-100..=-1).contains(&movement.energy_delta)
                || by_id.insert(movement.id.as_str(), movement).is_some()
                || !names.insert((movement.kind.as_str(), name_key(&movement.name)))
            {
                return Err("Invalid or ambiguous combat move");
            }
        }
        let mut forms = BTreeSet::new();
        let mut pair_count = 0usize;
        for pool in &self.pools {
            pair_count = pair_count.saturating_add(
                pool.fast_move_ids
                    .len()
                    .saturating_mul(pool.charged_move_ids.len()),
            );
            if pair_count > 250_000 {
                return Err("Combat reference exceeds the move-pair calculation limit");
            }
            if !valid_form_id(&pool.form_id) || !forms.insert(pool.form_id.as_str()) {
                return Err("Invalid or duplicate ordinary combat move pool");
            }
            if pool.withheld_moves.len() > 200
                || pool
                    .withheld_moves
                    .iter()
                    .any(|value| !crate::reference::valid_text(value, 120))
                || pool.withheld_moves.iter().collect::<BTreeSet<_>>().len()
                    != pool.withheld_moves.len()
            {
                return Err("Invalid withheld combat pool evidence");
            }
            for (ids, kind) in [
                (&pool.fast_move_ids, "fast"),
                (&pool.charged_move_ids, "charged"),
            ] {
                if ids.len() > 100 {
                    return Err("Combat move pool exceeds the bounded limit");
                }
                let mut unique = BTreeSet::new();
                for id in ids {
                    if !unique.insert(id)
                        || !by_id
                            .get(id.as_str())
                            .is_some_and(|movement| movement.kind == kind)
                    {
                        return Err("Combat move pool has duplicate, missing or wrong-kind moves");
                    }
                }
            }
        }
        Ok(())
    }
}

fn moves(value: &Value, kind: &str) -> Result<Vec<CombatMove>> {
    let rows = value
        .as_array()
        .filter(|rows| !rows.is_empty() && rows.len() <= 2_000)
        .ok_or("Expected a bounded nonempty PvE move array")?;
    rows.iter()
        .map(|row| {
            let id = row["move_id"]
                .as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .filter(|id| *id > 0)
                .ok_or("Expected a positive PvE move ID")?;
            let movement = CombatMove {
                id: format!("move-{id}"),
                name: row["name"].as_str().ok_or("Missing PvE move name")?.into(),
                move_type: canonical_type(row["type"].as_str().ok_or("Missing PvE move type")?)?,
                kind: kind.into(),
                power: row["power"]
                    .as_f64()
                    .ok_or("Missing numeric PvE move power")?,
                energy_delta: row["energy_delta"]
                    .as_i64()
                    .and_then(|value| i16::try_from(value).ok())
                    .ok_or("Invalid signed PvE move energy")?,
                duration_ms: row["duration"]
                    .as_u64()
                    .and_then(|value| u32::try_from(value).ok())
                    .ok_or("Invalid PvE move duration in milliseconds")?,
            };
            if !valid_name(&movement.name)
                || !movement.power.is_finite()
                || !(0.0..=10_000.0).contains(&movement.power)
                || !(1..=60_000).contains(&movement.duration_ms)
                || !(-100..=100).contains(&movement.energy_delta)
            {
                return Err("Invalid raw PvE move values");
            }
            if kind == "fast" && (movement.energy_delta < 0 || movement.power > 1_000.0) {
                return Err("Unsupported raw fast-move mechanics");
            }
            Ok(movement)
        })
        .collect()
}

fn effectiveness(value: &Value) -> Result<BTreeMap<String, BTreeMap<String, f64>>> {
    let chart = value
        .as_object()
        .ok_or("Expected a combat type chart object")?;
    if chart.len() != TYPES.len() {
        return Err("Combat type chart must cover all canonical types");
    }
    let mut output = BTreeMap::new();
    for (attacking, row) in chart {
        let attacking = canonical_type(attacking)?;
        let row = row.as_object().ok_or("Expected combat type chart rows")?;
        if row.len() != TYPES.len() {
            return Err("Combat type chart must cover all canonical types");
        }
        let mut normalized = BTreeMap::new();
        for (defending, multiplier) in row {
            let defending = canonical_type(defending)?;
            // The provider's documentation uses decimal strings; current raw
            // responses use JSON numbers. Both are explicit supplied values.
            let multiplier = multiplier
                .as_f64()
                .or_else(|| multiplier.as_str().and_then(|raw| raw.parse::<f64>().ok()))
                .ok_or("Invalid supplied type effectiveness value")?;
            if normalized.insert(defending, multiplier).is_some() {
                return Err("Ambiguous canonical defending type");
            }
        }
        if output.insert(attacking, normalized).is_some() {
            return Err("Ambiguous canonical attacking type");
        }
    }
    Ok(output)
}

/// Missing supplemental endpoints do not invalidate an existing catalog-only
/// import. A complete combat input is validated strictly and never downgraded
/// silently after a malformed numeric value or unresolved current move name.
pub fn normalize(provider: &str, value: &Value) -> Result<Option<CombatReference>> {
    if provider != "pogoapi" || KEYS.iter().any(|key| value.get(key).is_none()) {
        return Ok(None);
    }
    let raw_pools = value["current_pokemon_moves"]
        .as_array()
        .filter(|rows| rows.len() <= 10_000)
        .ok_or("Expected a bounded current Pokémon move array")?;
    let requested_fast: BTreeSet<_> = raw_pools
        .iter()
        .filter(|row| row["form"].as_str() == Some("Normal"))
        .filter_map(|row| row["fast_moves"].as_array())
        .flatten()
        .filter_map(Value::as_str)
        .map(name_key)
        .collect();
    let mut normalized_moves = moves(&value["fast_moves"], "fast")?;
    let charged = moves(&value["charged_moves"], "charged")?;
    // Recover a misclassified positive-energy record only when an explicit
    // current Normal pool independently identifies that exact name as fast.
    normalized_moves.extend(
        charged
            .iter()
            .filter(|movement| {
                movement.energy_delta > 0
                    && movement.power <= 1_000.0
                    && requested_fast.contains(&name_key(&movement.name))
            })
            .cloned()
            .map(|mut movement| {
                movement.kind = "fast".into();
                movement
            }),
    );
    // The live provider includes non-spending placeholders/misclassified fast
    // attacks in charged_moves. Withhold them and dependent charged options;
    // never manufacture energy costs. Unsupported pools can remain empty.
    let withheld: BTreeSet<_> = charged
        .iter()
        .filter(|movement| movement.energy_delta >= 0 || movement.power > 1_000.0)
        .map(|movement| name_key(&movement.name))
        .collect();
    normalized_moves.extend(
        charged
            .into_iter()
            .filter(|movement| movement.energy_delta < 0 && movement.power <= 1_000.0),
    );
    normalized_moves.sort_by(|a, b| a.id.cmp(&b.id));
    let mut ids = BTreeSet::new();
    let mut name_counts = BTreeMap::new();
    for movement in &normalized_moves {
        if !ids.insert(&movement.id) {
            return Err("Duplicate PvE move identifier");
        }
        *name_counts
            .entry((movement.kind.clone(), name_key(&movement.name)))
            .or_insert(0usize) += 1;
    }
    // Some event/dynamic variants share a display name but differ in ID,
    // power or typing. A name-only species pool cannot identify the variant.
    let ambiguous: BTreeSet<_> = name_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(name, _)| name)
        .collect();
    normalized_moves
        .retain(|movement| !ambiguous.contains(&(movement.kind.clone(), name_key(&movement.name))));
    let mut names = BTreeMap::new();
    for movement in &normalized_moves {
        if names
            .insert(
                (movement.kind.as_str(), name_key(&movement.name)),
                movement.id.clone(),
            )
            .is_some()
        {
            return Err("Ambiguous current PvE move name");
        }
    }
    let raw_pools = value["current_pokemon_moves"]
        .as_array()
        .filter(|rows| rows.len() <= 10_000)
        .ok_or("Expected a bounded current Pokémon move array")?;
    let mut pools = Vec::new();
    for row in raw_pools {
        // No omitted form, Shadow, Purified, regional or suffix inference.
        if row["form"].as_str() != Some("Normal") {
            continue;
        }
        let dex = row["pokemon_id"]
            .as_u64()
            .and_then(|id| u16::try_from(id).ok())
            .filter(|id| *id > 0)
            .ok_or("Invalid National Dex in current move pool")?;
        let resolve = |key: &str, kind: &str| -> Result<(Vec<String>, Vec<String>)> {
            let rows = row[key]
                .as_array()
                .filter(|rows| rows.len() <= 100)
                .ok_or("Expected a bounded array of current move names")?;
            let mut ids = BTreeSet::new();
            let mut seen = BTreeSet::new();
            let mut missing = Vec::new();
            for name in rows {
                let name = name.as_str().ok_or("Expected current move names")?;
                if !valid_name(name) {
                    return Err("Invalid current move name");
                }
                if !seen.insert(name_key(name)) {
                    return Err("Duplicate current move name in pool");
                }
                if kind == "charged" && withheld.contains(&name_key(name)) {
                    missing.push(format!("{kind}: {name}"));
                    continue;
                }
                if ambiguous.contains(&(kind.to_owned(), name_key(name))) {
                    missing.push(format!("{kind}: {name}"));
                    continue;
                }
                let Some(id) = names.get(&(kind, name_key(name))) else {
                    if names.keys().any(|(_, key)| *key == name_key(name)) {
                        return Err("Current move pool refers to a wrong-kind move");
                    }
                    missing.push(format!("{kind}: {name}"));
                    continue;
                };
                if !ids.insert(id.clone()) {
                    return Err("Duplicate current move in pool");
                }
            }
            Ok((ids.into_iter().collect(), missing))
        };
        let (fast_move_ids, mut withheld_moves) = resolve("fast_moves", "fast")?;
        let (charged_move_ids, charged_missing) = resolve("charged_moves", "charged")?;
        withheld_moves.extend(charged_missing);
        withheld_moves.sort();
        pools.push(CombatPool {
            form_id: format!("dex-{dex}-normal"),
            fast_move_ids,
            charged_move_ids,
            withheld_moves,
        });
        // elite_fast_moves and elite_charged_moves are deliberately not added
        // to current availability. No exclusive-move acquisition is assumed.
    }
    pools.sort_by(|a, b| a.form_id.cmp(&b.form_id));
    let normalized = CombatReference {
        version: VERSION.into(),
        moves: normalized_moves,
        pools,
        effectiveness: effectiveness(&value["type_effectiveness"])?,
    };
    normalized.validate()?;
    Ok(Some(normalized))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn source() -> Value {
        // Original fictional moves, species IDs and effectiveness values. This
        // all-neutral chart tests shape/joins, not real type matchups.
        let chart: BTreeMap<_, _> = TYPES
            .iter()
            .map(|attacking| {
                (
                    *attacking,
                    TYPES
                        .iter()
                        .map(|defending| (*defending, 1.0))
                        .collect::<BTreeMap<_, _>>(),
                )
            })
            .collect();
        json!({
            "fast_moves":[{"move_id":910001,"name":"Sample Spark","type":"Electric","power":8,"energy_delta":10,"duration":1000}],
            "charged_moves":[{"move_id":910002,"name":"Sample Burst","type":"Normal","power":40,"energy_delta":-50,"duration":2000}],
            "current_pokemon_moves":[
                {"pokemon_id":60000,"form":"Normal","fast_moves":["sample spark"],"charged_moves":["Sample Burst"],"elite_fast_moves":["Unmapped Exclusive"],"elite_charged_moves":["Unmapped Legacy"]},
                {"pokemon_id":60000,"form":"Shadow","fast_moves":["Not Imported"],"charged_moves":[]},
                {"pokemon_id":60001,"fast_moves":["Not Imported"],"charged_moves":[]}
            ],
            "type_effectiveness":chart
        })
    }

    #[test]
    fn normalizes_current_normal_pools_without_exclusive_or_variant_inference() {
        let normalized = normalize("pogoapi", &source()).unwrap().unwrap();
        assert_eq!(normalized.version, VERSION);
        assert_eq!(normalized.pools.len(), 1);
        assert_eq!(normalized.pools[0].form_id, "dex-60000-normal");
        assert_eq!(normalized.pools[0].fast_move_ids, ["move-910001"]);
        assert_eq!(normalized.pools[0].charged_move_ids, ["move-910002"]);
        assert_eq!(normalized.moves[0].move_type, "electric");
        assert_eq!(normalized.moves[0].energy_delta, 10);
        assert_eq!(normalized.moves[1].energy_delta, -50);
        assert_eq!(normalized.moves[1].duration_ms, 2000);
    }

    #[test]
    fn old_and_partial_imports_remain_catalog_only() {
        assert!(normalize("pogoapi", &json!({})).unwrap().is_none());
        let mut partial = source();
        partial
            .as_object_mut()
            .unwrap()
            .remove("type_effectiveness");
        assert!(normalize("pogoapi", &partial).unwrap().is_none());
        assert!(normalize("pvpoke", &source()).unwrap().is_none());
        assert!(normalize("pokeminers", &source()).unwrap().is_none());
        let mut malformed = source();
        malformed["fast_moves"] = Value::Null;
        assert!(normalize("pogoapi", &malformed).is_err());
    }

    #[test]
    fn rejects_invalid_values_missing_chart_cells_and_withholds_ambiguous_names() {
        for key in ["type", "power", "energy_delta", "duration", "move_id"] {
            let mut invalid = source();
            invalid["fast_moves"][0][key] = json!(-5);
            assert!(normalize("pogoapi", &invalid).is_err(), "{key}");
        }
        let mut invalid = source();
        invalid["fast_moves"][0]["type"] = json!("Fictional");
        assert!(normalize("pogoapi", &invalid).is_err());
        let mut invalid = source();
        invalid["type_effectiveness"]["fire"]
            .as_object_mut()
            .unwrap()
            .remove("ice");
        assert!(normalize("pogoapi", &invalid).is_err());
        let mut invalid = source();
        invalid["type_effectiveness"]["fire"]["ice"] = json!("NaN");
        assert!(normalize("pogoapi", &invalid).is_err());
        let mut invalid = source();
        let mut duplicate = invalid["fast_moves"][0].clone();
        duplicate["move_id"] = json!(910003);
        duplicate["name"] = json!("SAMPLE SPARK");
        invalid["fast_moves"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        let withheld = normalize("pogoapi", &invalid).unwrap().unwrap();
        assert!(withheld.pools[0].fast_move_ids.is_empty());
    }

    #[test]
    fn rejects_broken_duplicate_and_wrong_kind_move_pool_links() {
        for names in [
            json!(["Sample Burst"]),
            json!(["Sample Spark", "sample spark"]),
        ] {
            let mut invalid = source();
            invalid["current_pokemon_moves"][0]["fast_moves"] = names;
            assert!(normalize("pogoapi", &invalid).is_err());
        }
        let mut incomplete = source();
        incomplete["current_pokemon_moves"][0]["fast_moves"] = json!(["Unknown"]);
        let partial = normalize("pogoapi", &incomplete).unwrap().unwrap();
        assert!(partial.pools[0].fast_move_ids.is_empty());
        assert_eq!(partial.pools[0].withheld_moves, ["fast: Unknown"]);
        let mut invalid = source();
        let duplicate = invalid["current_pokemon_moves"][0].clone();
        invalid["current_pokemon_moves"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(normalize("pogoapi", &invalid).is_err());
        let mut reference = normalize("pogoapi", &source()).unwrap().unwrap();
        reference.pools[0].charged_move_ids = vec!["move-910001".into()];
        assert!(reference.validate().is_err());
    }

    #[test]
    fn explicit_decimal_chart_strings_and_input_order_have_stable_output() {
        let mut value = source();
        value["type_effectiveness"]["fire"]["ice"] = json!("1.25");
        let normalized = normalize("pogoapi", &value).unwrap().unwrap();
        assert_eq!(normalized.effectiveness["fire"]["ice"], 1.25);
        value["current_pokemon_moves"]
            .as_array_mut()
            .unwrap()
            .reverse();
        assert_eq!(normalize("pogoapi", &value).unwrap().unwrap(), normalized);
        let bytes = serde_json::to_vec(&normalized).unwrap();
        let decoded: CombatReference = serde_json::from_slice(&bytes).unwrap();
        decoded.validate().unwrap();
        assert_eq!(decoded, normalized);
    }

    #[test]
    fn preserves_explicit_gaps_and_recovers_only_independently_declared_fast_moves() {
        let mut value = source();
        value["charged_moves"].as_array_mut().unwrap().extend([
            json!({"move_id":910004,"name":"Sample Misclassified","type":"Normal","power":4,"energy_delta":5,"duration":500}),
            json!({"move_id":910005,"name":"Sample Free","type":"Normal","power":0,"energy_delta":0,"duration":1000}),
            json!({"move_id":910006,"name":"Sample Special","type":"Normal","power":9000,"energy_delta":-25,"duration":1000}),
        ]);
        value["current_pokemon_moves"][0]["fast_moves"] =
            json!(["Sample Spark", "Sample Misclassified"]);
        value["current_pokemon_moves"][0]["charged_moves"] = json!([
            "Sample Burst",
            "Sample Free",
            "Sample Special",
            "Sample Unmapped"
        ]);
        let normalized = normalize("pogoapi", &value).unwrap().unwrap();
        assert_eq!(normalized.pools[0].fast_move_ids.len(), 2);
        assert_eq!(normalized.pools[0].charged_move_ids.len(), 1);
        assert_eq!(normalized.pools[0].withheld_moves.len(), 3);
        assert_eq!(
            normalized
                .moves
                .iter()
                .find(|movement| movement.name == "Sample Misclassified")
                .unwrap()
                .kind,
            "fast"
        );
        let mut invalid = source();
        let duplicate = invalid["fast_moves"][0].clone();
        invalid["fast_moves"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(normalize("pogoapi", &invalid).is_err());
    }
}
