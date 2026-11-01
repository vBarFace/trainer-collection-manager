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
    if paths.is_empty() || paths.len() > 3 || provider != "pogoapi" && paths.len() != 1 {
        return Err("Select one reference JSON, or up to three PoGoAPI endpoint responses");
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
        if value.is_object() {
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
}
