//! Bounded adapters for user-selected local provider files. No network or assets.
//! Identity namespaces deliberately differ from Task 007 reviewed mappings.
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};

pub struct Normalized {
    pub catalog: Value,
    pub cp_multipliers: Vec<(f64, f64)>,
    pub coverage: Vec<String>,
    pub limitations: Vec<String>,
}

type Result<T> = std::result::Result<T, &'static str>;

struct Row {
    dex: u16,
    name: String,
    stats: Value,
    types: Value,
}

fn positive(value: &Value) -> Result<u16> {
    value
        .as_u64()
        .and_then(|n| u16::try_from(n).ok())
        .filter(|n| *n > 0)
        .ok_or("Expected a positive integer GO stat or National Dex number")
}

fn text(value: &Value) -> Result<String> {
    let value = value.as_str().ok_or("Expected provider text")?;
    if !crate::reference::valid_text(value, 100) {
        return Err("Invalid provider text");
    }
    Ok(value.to_owned())
}

fn types(value: &Value) -> Result<Value> {
    if value.is_null() {
        return Ok(Value::Null);
    }
    let values = value.as_array().ok_or("Expected a type array")?;
    let mut output = Vec::new();
    for value in values {
        let value = value
            .as_str()
            .ok_or("Expected a type name")?
            .to_ascii_lowercase();
        let value = value
            .strip_prefix("pokemon_type_")
            .unwrap_or(&value)
            .to_owned();
        if value == "none" && output.len() == 1 {
            continue;
        }
        if ![
            "normal", "fire", "water", "electric", "grass", "ice", "fighting", "poison", "ground",
            "flying", "psychic", "bug", "rock", "ghost", "dragon", "dark", "steel", "fairy",
        ]
        .contains(&value.as_str())
            || output.contains(&value)
        {
            return Err("Unsupported or duplicate GO type");
        }
        output.push(value);
    }
    if output.is_empty() || output.len() > 2 {
        return Err("Expected one or two GO types");
    }
    Ok(json!(output))
}

fn stats(a: &Value, d: &Value, s: &Value) -> Result<Value> {
    Ok(json!({"attack":positive(a)?,"defense":positive(d)?,"stamina":positive(s)?}))
}

fn pvpoke(value: &Value) -> Result<Vec<Row>> {
    let source = value
        .as_array()
        .or_else(|| value.get("pokemon").and_then(Value::as_array))
        .ok_or("PvPoke needs pokemon.json or gamemaster.json with a pokemon array")?;
    let mut by_dex: BTreeMap<u16, Vec<Row>> = BTreeMap::new();
    for row in source {
        let name = text(&row["speciesName"])?;
        let id = text(&row["speciesId"])?;
        // Never collapse simulator variants into individual properties or guess a
        // default for named forms. Ambiguous ordinary entries are withheld below.
        if name.contains(['(', ')'])
            || id.ends_with("_shadow")
            || id.contains("_mega")
            || row["tags"].as_array().is_some_and(|tags| {
                tags.iter()
                    .any(|v| matches!(v.as_str(), Some("shadow" | "mega")))
            })
        {
            continue;
        }
        let dex = positive(&row["dex"])?;
        by_dex.entry(dex).or_default().push(Row {
            dex,
            name,
            stats: stats(
                &row["baseStats"]["atk"],
                &row["baseStats"]["def"],
                &row["baseStats"]["hp"],
            )?,
            types: types(&row["types"])?,
        });
    }
    Ok(by_dex
        .into_values()
        .filter_map(|mut rows| if rows.len() == 1 { rows.pop() } else { None })
        .collect())
}

fn pogoapi(value: &Value) -> Result<(Vec<Row>, Vec<(f64, f64)>)> {
    let source = value["pokemon_stats"]
        .as_array()
        .ok_or("PoGoAPI needs an envelope containing pokemon_stats and pokemon_types arrays")?;
    let type_rows = value["pokemon_types"]
        .as_array()
        .ok_or("Missing PoGoAPI pokemon_types array")?;
    let mut type_by_dex = HashMap::new();
    for row in type_rows {
        if row["form"].as_str() != Some("Normal") {
            continue;
        }
        let dex = positive(&row["pokemon_id"])?;
        if type_by_dex.insert(dex, types(&row["type"])?).is_some() {
            return Err("Duplicate PoGoAPI Normal type record");
        }
    }
    let mut rows = Vec::new();
    for row in source {
        if row["form"].as_str() != Some("Normal") {
            continue;
        }
        let dex = positive(&row["pokemon_id"])?;
        rows.push(Row {
            dex,
            name: text(&row["pokemon_name"])?,
            stats: stats(
                &row["base_attack"],
                &row["base_defense"],
                &row["base_stamina"],
            )?,
            types: type_by_dex.get(&dex).cloned().unwrap_or(Value::Null),
        });
    }
    let mut cpm = Vec::new();
    if let Some(table) = value.get("cp_multiplier") {
        for entry in table
            .as_array()
            .ok_or("PoGoAPI cp_multiplier must be an array")?
        {
            let level = entry["level"].as_f64().ok_or("Invalid CP level")?;
            let multiplier = entry["multiplier"]
                .as_f64()
                .ok_or("Invalid CP multiplier")?;
            // Provider documentation explicitly warns that higher half-levels
            // are approximate. They cannot support verified investment advice.
            if level <= 40.0 {
                cpm.push((level, multiplier));
            }
        }
    }
    Ok((rows, cpm))
}

fn pokeminers(value: &Value) -> Result<(Vec<Row>, Vec<(f64, f64)>)> {
    let source = value
        .as_array()
        .ok_or("PokeMiners latest.json must be a template array")?;
    let mut rows = Vec::new();
    let mut cpm = Vec::new();
    for template in source {
        let data = template.get("data").unwrap_or(template);
        if let Some(levels) = data.get("playerLevel").and_then(|v| v.get("cpMultiplier")) {
            if !cpm.is_empty() {
                return Err("Duplicate PokeMiners CP multiplier table");
            }
            for (index, multiplier) in levels
                .as_array()
                .ok_or("Malformed PokeMiners CP multiplier table")?
                .iter()
                .enumerate()
            {
                let level = index as f64 + 1.0;
                if level <= 50.0 {
                    cpm.push((
                        level,
                        multiplier.as_f64().ok_or("Invalid PokeMiners multiplier")?,
                    ));
                }
            }
        }
        let Some(settings) = data.get("pokemonSettings") else {
            continue;
        };
        let template_id = template
            .get("templateId")
            .or_else(|| data.get("templateId"))
            .and_then(Value::as_str)
            .ok_or("Missing PokeMiners templateId")?;
        let Some(rest) = template_id.strip_prefix('V') else {
            continue;
        };
        let Some((digits, enum_name)) = rest.split_once("_POKEMON_") else {
            continue;
        };
        let dex: u16 = digits
            .parse()
            .map_err(|_| "Invalid PokeMiners National Dex template")?;
        if dex == 0 {
            return Err("Invalid National Dex number");
        }
        let pokemon_id = settings["pokemonId"]
            .as_str()
            .ok_or("Missing PokeMiners pokemonId")?;
        // Exact template suffix equality excludes regional/costume variants.
        if enum_name != pokemon_id {
            continue;
        }
        if settings.get("form").is_some_and(|form| {
            !form.is_null()
                && form.as_str() != Some("UNSET")
                && form.as_str() != Some("NORMAL")
                && form.as_u64() != Some(0)
        }) {
            continue;
        }
        let name = pokemon_id
            .split('_')
            .map(|part| {
                let lower = part.to_ascii_lowercase();
                let mut chars = lower.chars();
                chars
                    .next()
                    .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join(" ");
        let mut type_values = vec![settings["type"].clone()];
        if let Some(second) = settings.get("type2") {
            if !second.is_null() {
                type_values.push(second.clone());
            }
        }
        rows.push(Row {
            dex,
            name,
            stats: stats(
                &settings["stats"]["baseAttack"],
                &settings["stats"]["baseDefense"],
                &settings["stats"]["baseStamina"],
            )?,
            types: types(&Value::Array(type_values))?,
        });
    }
    Ok((rows, cpm))
}

pub fn normalize(
    provider: &str,
    bytes: &[u8],
    revision: &str,
    retrieved_at: &str,
) -> Result<Normalized> {
    if bytes.len() > 64 * 1024 * 1024 {
        return Err("Provider source exceeds 64 MiB");
    }
    if !crate::reference::valid_text(revision, 100)
        || retrieved_at.len() != 10
        || !retrieved_at.bytes().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                b == b'-'
            } else {
                b.is_ascii_digit()
            }
        })
    {
        return Err("A source revision and YYYY-MM-DD retrieval date are required");
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| "Malformed provider JSON")?;
    let (rows, mut cpm, source_url, rights_url, license) = match provider {
        "pvpoke" => (
            pvpoke(&value)?,
            vec![],
            "https://github.com/pvpoke/pvpoke",
            "https://github.com/pvpoke/pvpoke/blob/master/LICENSE",
            "MIT software; underlying game-data rights unverified",
        ),
        "pokeminers" => {
            let (rows, cpm) = pokeminers(&value)?;
            (
                rows,
                cpm,
                "https://github.com/PokeMiners/game_masters",
                "https://github.com/PokeMiners/game_masters#disclaimer",
                "No dataset redistribution grant verified; educational-use disclaimer",
            )
        }
        "pogoapi" => {
            let (rows, cpm) = pogoapi(&value)?;
            (
                rows,
                cpm,
                "https://pogoapi.net/documentation/",
                "https://pogoapi.net/Legal/",
                "Provider fair-use notice; downstream game-data rights unverified",
            )
        }
        _ => return Err("Unsupported reference provider"),
    };
    if rows.is_empty() || rows.len() > 10000 {
        return Err("No supported unambiguous ordinary forms found, or source is too large");
    }
    let mut species = BTreeMap::new();
    let mut forms = Vec::new();
    for row in rows {
        let id = format!("dex-{}", row.dex);
        if species
            .insert(
                row.dex,
                json!({"id":id,"dexNumber":row.dex,"name":row.name,"aliases":[]}),
            )
            .is_some()
        {
            return Err("Duplicate ordinary species; explicit form mappings are required");
        }
        forms.push(json!({"id":format!("{id}-normal"),"speciesId":id,"name":"Normal","aliases":[],"types":row.types,"baseStats":row.stats,"evolutions":null}));
    }
    forms.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    cpm.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut previous = (0.0, 0.0);
    for &(level, multiplier) in &cpm {
        if !(1.0..=50.0).contains(&level)
            || (level * 2.0).fract() != 0.0
            || !(0.0..1.0).contains(&multiplier)
            || level <= previous.0
            || multiplier <= previous.1
        {
            return Err("Invalid, duplicate, or non-monotonic CP multiplier table");
        }
        previous = (level, multiplier);
    }
    let catalog = json!({"schemaVersion":1,"dataset":{
        "datasetId":format!("local-{provider}-normal-1"),"version":revision,
        "sourceUrl":source_url,"upstreamRevision":revision,"license":license,
        "notice":"User-selected local data only. Original application MIT license does not cover underlying Pokémon GO data. Public accessibility is not a redistribution grant.",
        "retrievedAt":retrieved_at,"rightsEvidenceUrl":rights_url
    },"species":species.into_values().collect::<Vec<_>>(),"forms":forms});
    crate::reference::Catalog::load(
        &serde_json::to_vec(&catalog).map_err(|_| "Invalid normalized catalog")?,
    )?;
    let mut limitations=vec![
        "Bounded ordinary forms only; regional, named, costume, Mega, Shadow and ambiguous forms are not normalized. Individual variants remain unknown.".into(),
        "Moves, evolution eligibility/costs, availability, rarity and battle rankings are unsupported by this adapter. Missing relationships are unknown, never no-evolution claims.".into(),
        "Provider revision is user supplied; the raw-file SHA-256 identifies actual imported bytes. Rights uncertainty remains unresolved; do not redistribute downloaded datasets.".into(),
    ];
    if provider == "pokeminers" {
        limitations.push("PokeMiners labels derive from source enum identifiers; special names may need explicit manual identity confirmation. Integer-level CP multipliers only; no guessed half-level interpolation.".into());
    }
    if provider == "pogoapi" {
        limitations.push("Only explicitly named Normal form rows are imported. CP multipliers above level 40 are withheld because provider documentation warns about approximation.".into());
    }
    if cpm.is_empty() {
        limitations.push("No supported CP multiplier table in this source; PvP level/stat-product calculations are unavailable.".into());
    }
    Ok(Normalized {
        catalog,
        cp_multipliers: cpm,
        coverage: vec![
            "National Dex identifiers and ordinary-form identity".into(),
            "GO base Attack, Defense and Stamina".into(),
            "Types where present".into(),
        ],
        limitations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fictional_pogoapi_normal_only_and_bounded_cpm() {
        let source = json!({"pokemon_stats":[{"pokemon_id":61001,"pokemon_name":"Emberwit","form":"Normal","base_attack":120,"base_defense":100,"base_stamina":130},{"pokemon_id":61001,"pokemon_name":"Emberwit","form":"Coastal","base_attack":130,"base_defense":100,"base_stamina":130}],"pokemon_types":[{"pokemon_id":61001,"form":"Normal","type":["Fire"]}],"cp_multiplier":[{"level":1,"multiplier":0.1},{"level":40,"multiplier":0.8},{"level":40.5,"multiplier":0.81}]});
        let output = normalize(
            "pogoapi",
            &serde_json::to_vec(&source).unwrap(),
            "original-fiction-1",
            "2026-11-01",
        )
        .unwrap();
        assert_eq!(output.catalog["forms"].as_array().unwrap().len(), 1);
        assert_eq!(output.catalog["forms"][0]["types"], json!(["fire"]));
        assert!(output.catalog["forms"][0]["evolutions"].is_null());
        assert_eq!(output.cp_multipliers, vec![(1.0, 0.1), (40.0, 0.8)]);
    }
    #[test]
    fn fictional_pvpoke_rejects_ambiguous_and_variant_entries() {
        let row = json!({"dex":61001,"speciesName":"Emberwit","speciesId":"emberwit","baseStats":{"atk":120,"def":100,"hp":130},"types":["fire","none"]});
        let mut shadow = row.clone();
        shadow["speciesId"] = json!("emberwit_shadow");
        shadow["speciesName"] = json!("Emberwit (Shadow)");
        let output = normalize(
            "pvpoke",
            &serde_json::to_vec(&json!([row.clone(), shadow])).unwrap(),
            "original-fiction-1",
            "2026-11-01",
        )
        .unwrap();
        assert_eq!(output.catalog["forms"].as_array().unwrap().len(), 1);
        assert!(normalize(
            "pvpoke",
            &serde_json::to_vec(&json!([row.clone(), row])).unwrap(),
            "original-fiction-1",
            "2026-11-01"
        )
        .is_err());
    }
    #[test]
    fn fictional_pokeminers_template_identity_and_integer_table() {
        let source = json!([{"templateId":"V61001_POKEMON_EMBERWIT","data":{"pokemonSettings":{"pokemonId":"EMBERWIT","type":"POKEMON_TYPE_FIRE","type2":"POKEMON_TYPE_NONE","stats":{"baseAttack":120,"baseDefense":100,"baseStamina":130}}}},{"templateId":"PLAYER_LEVEL_SETTINGS","data":{"playerLevel":{"cpMultiplier":[0.1,0.2]}}}]);
        let output = normalize(
            "pokeminers",
            &serde_json::to_vec(&source).unwrap(),
            "original-fiction-1",
            "2026-11-01",
        )
        .unwrap();
        assert_eq!(output.catalog["species"][0]["name"], "Emberwit");
        assert_eq!(output.catalog["forms"][0]["types"], json!(["fire"]));
        assert_eq!(output.cp_multipliers, vec![(1.0, 0.1), (2.0, 0.2)]);
    }
    #[test]
    fn malformed_unknown_and_duplicate_sources_fail_closed() {
        assert!(normalize("unknown", b"{}", "revision", "2026-11-01").is_err());
        assert!(normalize("pogoapi", b"{}", "revision", "2026-11-01").is_err());
        assert!(normalize("pvpoke", b"not json", "revision", "2026-11-01").is_err());
    }
}
