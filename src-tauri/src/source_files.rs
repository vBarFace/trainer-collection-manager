//! Read only native-picker-selected files. Paths and filenames never enter
//! persisted source artifacts or user-facing error messages.
use crate::reference_store::{digest, MAX_SOURCE_BYTES};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceArtifact {
    pub kind: String,
    pub source_url: String,
    pub sha256: String,
    pub upstream_revision: String,
    pub retrieved_at: String,
}

pub struct SourceInput {
    pub bytes: Vec<u8>,
    pub artifacts: Vec<SourceArtifact>,
}

type Result<T> = std::result::Result<T, &'static str>;

fn artifact(kind: &str, url: &str, bytes: &[u8], revision: &str, date: &str) -> SourceArtifact {
    SourceArtifact {
        kind: kind.into(),
        source_url: url.into(),
        sha256: digest(bytes),
        upstream_revision: revision.into(),
        retrieved_at: date.into(),
    }
}

fn row_kind(row: &Value) -> Result<&'static str> {
    if !row.is_object() {
        return Err("PoGoAPI responses must contain object records");
    }
    let candidates = [
        ("pokemon_stats", row.get("base_attack").is_some()),
        (
            "pokemon_types",
            row.get("type").is_some() && row.get("pokemon_id").is_some(),
        ),
        (
            "cp_multiplier",
            row.get("level").is_some() && row.get("multiplier").is_some(),
        ),
        (
            "current_pokemon_moves",
            row.get("fast_moves").is_some() && row.get("pokemon_id").is_some(),
        ),
        (
            "pokemon_evolutions",
            row.get("evolutions").is_some() && row.get("pokemon_id").is_some(),
        ),
        (
            "trainer_moves",
            row.get("move_id").is_some() && row.get("turn_duration").is_some(),
        ),
        (
            "pve_moves",
            row.get("move_id").is_some() && row.get("turn_duration").is_none(),
        ),
    ];
    let mut matches = candidates.iter().filter(|(_, applies)| *applies);
    let kind = matches
        .next()
        .map(|(kind, _)| *kind)
        .ok_or("Unsupported PoGoAPI endpoint response fields")?;
    if matches.next().is_some() {
        return Err("Ambiguous PoGoAPI endpoint response fields");
    }
    Ok(kind)
}

fn endpoint(kind: &str) -> &'static str {
    match kind {
        "pokemon_stats" => "https://pogoapi.net/api/v1/pokemon_stats.json",
        "pokemon_types" => "https://pogoapi.net/api/v1/pokemon_types.json",
        "cp_multiplier" => "https://pogoapi.net/api/v1/cp_multiplier.json",
        "fast_moves" => "https://pogoapi.net/api/v1/fast_moves.json",
        "charged_moves" => "https://pogoapi.net/api/v1/charged_moves.json",
        "current_pokemon_moves" => "https://pogoapi.net/api/v1/current_pokemon_moves.json",
        "type_effectiveness" => "https://pogoapi.net/api/v1/type_effectiveness.json",
        "pokemon_evolutions" => "https://pogoapi.net/api/v1/pokemon_evolutions.json",
        "pokemon_powerup_requirements" => {
            "https://pogoapi.net/api/v1/pokemon_powerup_requirements.json"
        }
        "pvp_fast_moves" => "https://pogoapi.net/api/v1/pvp_fast_moves.json",
        "pvp_charged_moves" => "https://pogoapi.net/api/v1/pvp_charged_moves.json",
        _ => "https://pogoapi.net/documentation/",
    }
}

/// A PoGoAPI envelope preserves its exact selected bytes. Separate endpoint
/// responses are combined deterministically and retain individual raw digests.
/// The supplied upstream revision is a user declaration, not verified remotely.
pub fn read(
    provider: &str,
    paths: &[PathBuf],
    revision: &str,
    retrieved_at: &str,
) -> Result<SourceInput> {
    if !matches!(provider, "pvpoke" | "pokeminers" | "pogoapi" | "snapshot") {
        return Err("Unsupported reference provider");
    }
    if paths.is_empty() || paths.len() > 11 || provider != "pogoapi" && paths.len() != 1 {
        return Err("Select one reference JSON, or up to eleven PoGoAPI endpoint responses");
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
        return Err("Supply a source revision and YYYY-MM-DD retrieval date");
    }

    // Check aggregate advertised size before allocating source buffers. The
    // capped reads also catch growth after metadata inspection.
    let mut files = Vec::new();
    let mut advertised = 0u64;
    for path in paths {
        let file = File::open(path).map_err(|_| "Unable to read a selected reference file")?;
        let metadata = file
            .metadata()
            .map_err(|_| "Unable to inspect a selected reference file")?;
        if !metadata.is_file() {
            return Err("Select ordinary reference JSON files");
        }
        advertised = advertised
            .checked_add(metadata.len())
            .ok_or("Reference sources exceed the aggregate 64 MiB limit")?;
        if advertised > MAX_SOURCE_BYTES {
            return Err("Reference sources exceed the aggregate 64 MiB limit");
        }
        files.push(file);
    }
    let mut raw_files = Vec::new();
    let mut consumed = 0u64;
    for file in files {
        let mut bytes = Vec::new();
        file.take(MAX_SOURCE_BYTES - consumed + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Unable to read a selected reference file")?;
        consumed += bytes.len() as u64;
        if consumed > MAX_SOURCE_BYTES {
            return Err("Reference sources exceed the aggregate 64 MiB limit");
        }
        raw_files.push(bytes);
    }
    if provider != "pogoapi" {
        let bytes = raw_files.pop().ok_or("No reference file selected")?;
        let url = match provider {
            "pvpoke" => "https://github.com/pvpoke/pvpoke",
            "pokeminers" => "https://github.com/PokeMiners/game_masters",
            _ => "https://github.com/vBarFace/trainer-collection-manager/blob/main/docs/development/reference-data.md",
        };
        return Ok(SourceInput {
            artifacts: vec![artifact(provider, url, &bytes, revision, retrieved_at)],
            bytes,
        });
    }

    let mut envelope = BTreeMap::new();
    let mut artifacts = Vec::new();
    let single = raw_files.len() == 1;
    for bytes in raw_files {
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|_| "Malformed PoGoAPI JSON response")?;
        if value.get("pokemon_stats").is_some() {
            if !single {
                return Err(
                    "Select one PoGoAPI envelope or separate raw endpoint arrays; do not mix them",
                );
            }
            return Ok(SourceInput {
                artifacts: vec![artifact(
                    "bundle",
                    "https://pogoapi.net/documentation/",
                    &bytes,
                    revision,
                    retrieved_at,
                )],
                bytes,
            });
        }
        if value.is_object() {
            if value.as_object().is_some_and(|rows| {
                !rows.is_empty()
                    && rows.values().all(|row| {
                        row.get("current_level").is_some()
                            && row.get("level_after_powering").is_some()
                    })
            }) {
                if envelope
                    .insert("pokemon_powerup_requirements", value)
                    .is_some()
                {
                    return Err("Duplicate PoGoAPI Power Up requirements selected");
                }
                artifacts.push(artifact(
                    "pokemon_powerup_requirements",
                    endpoint("pokemon_powerup_requirements"),
                    &bytes,
                    revision,
                    retrieved_at,
                ));
                continue;
            }
            if !value.as_object().is_some_and(|chart| {
                chart.len() == 18
                    && chart.iter().all(|(kind, row)| {
                        crate::combat_reference::TYPES.contains(&kind.to_ascii_lowercase().as_str())
                            && row.as_object().is_some_and(|row| row.len() == 18)
                    })
            }) {
                return Err(
                    "Unsupported PoGoAPI object; select a source envelope or full type chart",
                );
            }
            if envelope.insert("type_effectiveness", value).is_some() {
                return Err("Duplicate PoGoAPI type effectiveness response selected");
            }
            artifacts.push(artifact(
                "type_effectiveness",
                endpoint("type_effectiveness"),
                &bytes,
                revision,
                retrieved_at,
            ));
            continue;
        }
        let rows = value
            .as_array()
            .filter(|rows| !rows.is_empty())
            .ok_or("Select nonempty PoGoAPI endpoint response arrays")?;
        let kind = row_kind(&rows[0])?;
        for row in rows {
            if row_kind(row)? != kind {
                return Err("Mixed PoGoAPI record kinds in one endpoint response");
            }
        }
        // Charged responses may contain misclassified fast entries. Inspect
        // every row rather than depend on provider ordering; the normalizer
        // records exclusions/recovery and never guesses energy costs.
        let kind = if kind == "trainer_moves" {
            if rows
                .iter()
                .any(|row| row["energy_delta"].as_i64().is_some_and(|v| v < 0))
            {
                "pvp_charged_moves"
            } else {
                "pvp_fast_moves"
            }
        } else if kind == "pve_moves" {
            if rows.iter().any(|row| {
                row.get("critical_chance").is_some()
                    || row["energy_delta"]
                        .as_i64()
                        .is_some_and(|energy| energy < 0)
            }) {
                "charged_moves"
            } else {
                "fast_moves"
            }
        } else {
            kind
        };
        if envelope.insert(kind, value).is_some() {
            return Err("Duplicate PoGoAPI endpoint response selected");
        }
        artifacts.push(artifact(
            kind,
            endpoint(kind),
            &bytes,
            revision,
            retrieved_at,
        ));
    }
    artifacts.sort_by(|a, b| a.kind.cmp(&b.kind));
    let bytes = serde_json::to_vec(&envelope).map_err(|_| "Unable to combine PoGoAPI responses")?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err("Combined PoGoAPI reference exceeds the 64 MiB limit");
    }
    Ok(SourceInput { bytes, artifacts })
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Samples(Vec<PathBuf>);
    impl Samples {
        fn new(contents: &[&[u8]]) -> Self {
            let unique = format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            let paths = contents
                .iter()
                .enumerate()
                .map(|(index, bytes)| {
                    let path = std::env::temp_dir()
                        .join(format!("private-reference-sample-{unique}-{index}.json"));
                    std::fs::write(&path, bytes).unwrap();
                    path
                })
                .collect();
            Self(paths)
        }
    }
    impl Drop for Samples {
        fn drop(&mut self) {
            for path in &self.0 {
                let _ = std::fs::remove_file(path);
            }
        }
    }
    const REVISION: &str = "original-fictional-response";
    const DATE: &str = "2026-11-01";

    #[test]
    fn separate_responses_are_order_independent_and_preserve_raw_digests_without_paths() {
        let stats = br#"[ {"pokemon_id":61001,"pokemon_name":"Emberwit","form":"Normal","base_attack":120,"base_defense":100,"base_stamina":130} ]"#;
        let types = br#"[{"pokemon_id":61001,"form":"Normal","type":["Fire"]}]"#;
        let multipliers = br#"[{"level":1,"multiplier":0.1}]"#;
        let samples = Samples::new(&[stats, types, multipliers]);
        let first = read("pogoapi", &samples.0, REVISION, DATE).unwrap();
        let reversed = samples.0.iter().rev().cloned().collect::<Vec<_>>();
        let second = read("pogoapi", &reversed, REVISION, DATE).unwrap();
        assert_eq!(first.bytes, second.bytes);
        assert_eq!(first.artifacts, second.artifacts);
        let stats_artifact = first
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == "pokemon_stats")
            .unwrap();
        assert_eq!(stats_artifact.sha256, digest(stats));
        assert_eq!(
            stats_artifact.source_url,
            "https://pogoapi.net/api/v1/pokemon_stats.json"
        );
        let json = serde_json::to_string(&first.artifacts).unwrap();
        assert!(!json.contains("private-reference-sample"));
        assert!(!json.contains("path"));
    }

    #[test]
    fn envelope_and_other_provider_single_files_preserve_exact_bytes() {
        let envelope = br#"{ "pokemon_stats":[], "pokemon_types":[] }"#;
        let samples = Samples::new(&[envelope]);
        let source = read("pogoapi", &samples.0, REVISION, DATE).unwrap();
        assert_eq!(source.bytes, envelope);
        assert_eq!(source.artifacts[0].kind, "bundle");
        assert_eq!(source.artifacts[0].sha256, digest(envelope));
        let source = read("pvpoke", &samples.0, REVISION, DATE).unwrap();
        assert_eq!(source.bytes, envelope);
    }

    #[test]
    fn duplicate_mixed_ambiguous_empty_and_bundle_plus_raw_fail() {
        for contents in [
            vec![
                br#"[{"base_attack":120}]"#.as_slice(),
                br#"[{"base_attack":130}]"#.as_slice(),
            ],
            vec![br#"[{"base_attack":120},{"level":1,"multiplier":0.1}]"#.as_slice()],
            vec![br#"[{"base_attack":120,"pokemon_id":61001,"type":["Fire"]}]"#.as_slice()],
            vec![b"[]".as_slice()],
            vec![b"{}".as_slice(), br#"[{"base_attack":120}]"#.as_slice()],
        ] {
            let samples = Samples::new(&contents);
            assert!(read("pogoapi", &samples.0, REVISION, DATE).is_err());
        }
    }

    #[test]
    fn aggregate_size_checked_before_reading_and_selection_is_bounded() {
        let samples = Samples::new(&[b"", b""]);
        File::options()
            .write(true)
            .open(&samples.0[0])
            .unwrap()
            .set_len(33 * 1024 * 1024)
            .unwrap();
        File::options()
            .write(true)
            .open(&samples.0[1])
            .unwrap()
            .set_len(32 * 1024 * 1024)
            .unwrap();
        assert!(
            matches!(read("pogoapi",&samples.0,REVISION,DATE),Err(message) if message.contains("aggregate"))
        );
        assert!(read("pvpoke", &samples.0, REVISION, DATE).is_err());
        assert!(read("pogoapi", &[], REVISION, DATE).is_err());
    }

    #[test]
    fn seven_raw_endpoints_preserve_type_chart_and_misclassified_move_evidence() {
        let chart: std::collections::BTreeMap<_, _> = crate::combat_reference::TYPES
            .iter()
            .map(|kind| {
                (
                    *kind,
                    crate::combat_reference::TYPES
                        .iter()
                        .map(|kind| (*kind, 1.0))
                        .collect::<std::collections::BTreeMap<_, _>>(),
                )
            })
            .collect();
        let chart = serde_json::to_vec(&chart).unwrap();
        let samples=Samples::new(&[
            br#"[{"pokemon_id":61001,"form":"Normal","pokemon_name":"Sampleton","base_attack":100,"base_defense":100,"base_stamina":100}]"#,
            br#"[{"pokemon_id":61001,"form":"Normal","type":["Normal"]}]"#,
            br#"[{"level":1,"multiplier":0.1}]"#,
            br#"[{"move_id":910001,"name":"Sample Tap","type":"Normal","power":10,"energy_delta":10,"duration":1000}]"#,
            br#"[{"move_id":910002,"name":"Sample Slam","type":"Normal","power":100,"energy_delta":-50,"duration":2000,"critical_chance":0},{"move_id":910003,"name":"Sample Fast","type":"Normal","power":4,"energy_delta":5,"duration":500}]"#,
            br#"[{"pokemon_id":61001,"form":"Normal","fast_moves":["Sample Tap","Sample Fast"],"charged_moves":["Sample Slam"]}]"#,
            &chart,
        ]);
        let first = read("pogoapi", &samples.0, REVISION, DATE).unwrap();
        assert_eq!(first.artifacts.len(), 7);
        let reversed = samples.0.iter().rev().cloned().collect::<Vec<_>>();
        assert_eq!(
            read("pogoapi", &reversed, REVISION, DATE).unwrap().bytes,
            first.bytes
        );
        let normalized =
            crate::source_adapter::normalize("pogoapi", &first.bytes, REVISION, DATE).unwrap();
        let combat = normalized.combat.unwrap();
        assert_eq!(combat.moves.len(), 3);
        assert_eq!(combat.pools[0].fast_move_ids.len(), 2);
        let mut charged: Value =
            serde_json::from_slice(&std::fs::read(&samples.0[4]).unwrap()).unwrap();
        charged.as_array_mut().unwrap().reverse();
        std::fs::write(&samples.0[4], serde_json::to_vec(&charged).unwrap()).unwrap();
        let reordered = read("pogoapi", &samples.0, REVISION, DATE).unwrap();
        assert_eq!(
            crate::source_adapter::normalize("pogoapi", &reordered.bytes, REVISION, DATE)
                .unwrap()
                .combat
                .unwrap(),
            combat
        );
        assert_eq!(
            first
                .artifacts
                .iter()
                .find(|artifact| artifact.kind == "type_effectiveness")
                .unwrap()
                .sha256,
            digest(&chart)
        );
    }

    #[test]
    fn preparation_and_trainer_endpoint_kinds_preserve_provenance_and_reject_duplicates() {
        let samples=Samples::new(&[
            br#"[{"pokemon_id":61001,"form":"Normal","evolutions":[{"pokemon_id":61002,"form":"Normal","candy_required":25}]}]"#,
            br#"{"39.5":{"current_level":39.5,"level_after_powering":40,"candy_to_upgrade":1,"xl_candy_to_upgrade":0,"stardust_to_upgrade":10}}"#,
            br#"[{"move_id":910001,"name":"Sample Tap","type":"Normal","power":4,"energy_delta":8,"turn_duration":2}]"#,
            br#"[{"move_id":910002,"name":"Sample Slam","type":"Normal","power":40,"energy_delta":-40,"turn_duration":1}]"#,
        ]);
        let source = read("pogoapi", &samples.0, REVISION, DATE).unwrap();
        let envelope: Value = serde_json::from_slice(&source.bytes).unwrap();
        for key in [
            "pokemon_evolutions",
            "pokemon_powerup_requirements",
            "pvp_fast_moves",
            "pvp_charged_moves",
        ] {
            assert!(envelope.get(key).is_some());
            assert!(source
                .artifacts
                .iter()
                .any(|a| a.kind == key && a.source_url.ends_with(&format!("{key}.json"))));
        }
        let mut duplicate = samples.0.clone();
        duplicate.push(samples.0[1].clone());
        assert!(read("pogoapi", &duplicate, REVISION, DATE).is_err());
        let too_many = std::iter::repeat_n(samples.0[0].clone(), 12).collect::<Vec<_>>();
        assert!(read("pogoapi", &too_many, REVISION, DATE).is_err());
    }
}
