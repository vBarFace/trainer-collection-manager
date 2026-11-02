//! Portable structured user facts only. No screenshot bytes, names, paths or
//! downloaded catalog documents are exported.
use crate::inventory::{InventoryRepository, Result, TagScope};
use serde_json::{json, Value};

impl InventoryRepository {
    pub fn structured_backup(&self) -> Result<Vec<u8>> {
        let transaction = self.connection.unchecked_transaction()?;
        let mut records = Vec::new();
        for pokemon in self.list()? {
            let tags=self.tags_for(&pokemon.id)?.into_iter().map(|tag|json!({"id":tag.id.as_i64().to_string(),"name":tag.name,"scope":match tag.scope{TagScope::Standard=>"standard",TagScope::Local=>"local"}})).collect::<Vec<_>>();
            let history = self.history(&pokemon.id)?;
            let pin:Option<(Option<String>,Option<String>,Option<String>)>=self.connection.query_row("SELECT dataset_id,species_id,form_id FROM pokemon_reference_identity WHERE pokemon_id=?1",[pokemon.id.as_str()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
            let pin=pin.map(|(dataset,species,form)|json!({"datasetId":dataset,"speciesId":species,"formId":form}));
            records.push(json!({
                "id":pokemon.id.as_str(),"displayName":pokemon.display_name,"cp":pokemon.cp.map(|v|v.to_string()),"ivs":pokemon.ivs,
                "strategicObservationRevision":pokemon.strategic_observation_revision.to_string(),"createdAt":pokemon.created_at,"updatedAt":pokemon.updated_at,
                "confirmedIdentity":pokemon.reference.identity,
                "referencePin":pin,
                "tags":tags,"details":self.individual_details(&pokemon.id)?,"confirmedHistory":history,
                "preparationResources":self.preparation_resources(&pokemon.id)?
            }));
        }
        let preferences: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM application_settings WHERE key='strategy-preferences'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        let roadmaps: Vec<Value> = {
            let mut query=self.connection.prepare("SELECT pokemon_id,context,report_json,previous_json,progress_json,revision,generated_at FROM pokemon_roadmaps ORDER BY pokemon_id")?;
            let rows=query.query_map([],|r|Ok(json!({"pokemonId":r.get::<_,String>(0)?,"context":r.get::<_,String>(1)?,"report":serde_json::from_str::<Value>(&r.get::<_,String>(2)?).unwrap_or(Value::Null),"previous":r.get::<_,Option<String>>(3)?.and_then(|v|serde_json::from_str::<Value>(&v).ok()),"progress":serde_json::from_str::<Value>(&r.get::<_,String>(4)?).unwrap_or(Value::Null),"revision":r.get::<_,i64>(5)?.to_string(),"generatedAt":r.get::<_,String>(6)?})))?;
            rows.collect::<rusqlite::Result<_>>()?
        };
        let now: String = self
            .connection
            .query_row("SELECT datetime('now')", [], |r| r.get(0))?;
        let tag_definitions=self.tags()?.into_iter().map(|tag|json!({"id":tag.id.as_i64().to_string(),"name":tag.name,"scope":match tag.scope{TagScope::Standard=>"standard",TagScope::Local=>"local"}})).collect::<Vec<_>>();
        let document = json!({"format":"tcm-backup-1","exportedAtUtc":now,"inventory":records,"tagDefinitions":tag_definitions,"preferences":preferences.and_then(|v|serde_json::from_str::<Value>(&v).ok()),"roadmaps":roadmaps,"notice":"Confirmed user data and derived plans only. Downloaded reference datasets and screenshots are excluded. Keep a separate private SQLite copy if full database recovery is required; backup import is not implemented."});
        let bytes = serde_json::to_vec_pretty(&document)
            .map_err(|_| crate::inventory::Error::InvalidInput("Unable to serialize backup"))?;
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(crate::inventory::Error::InvalidInput(
                "Structured backup exceeds 64 MiB; use a private database backup",
            ));
        }
        transaction.commit()?;
        Ok(bytes)
    }
}
use rusqlite::OptionalExtension;

pub fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("Missing output directory"))?;
    let temporary = parent.join(format!(
        ".tcm-backup-{}-{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::PokemonInput;
    #[test]
    fn export_excludes_catalogs_and_preserves_unavailable_pins_and_tag_library() {
        let path = std::env::temp_dir().join(format!(
            "tcm-backup-test-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut repo = InventoryRepository::open(&path).unwrap();
        let fixture = include_bytes!("../tests/fixtures/reference-synthetic.json");
        repo.activate_reference(fixture).unwrap();
        let pokemon = repo
            .create(&PokemonInput {
                display_name: "Private fictional nickname".into(),
                cp: Some(123),
                ivs: None,
            })
            .unwrap();
        repo.correct_identity(
            &pokemon.id,
            None,
            Some(crate::reference::Identity {
                species: "Testling".into(),
                form: Some("Normal".into()),
            }),
        )
        .unwrap();
        repo.create_custom_tag("Unassigned personal tag").unwrap();
        let mut changed: Value = serde_json::from_slice(fixture).unwrap();
        let old_namespace = changed["dataset"]["datasetId"].clone();
        changed["dataset"]["datasetId"] = Value::from("other-provider");
        repo.activate_reference(&serde_json::to_vec(&changed).unwrap())
            .unwrap();
        assert_eq!(
            repo.get(&pokemon.id).unwrap().reference.state,
            crate::reference::ResolutionState::Unavailable
        );
        let export: Value = serde_json::from_slice(&repo.structured_backup().unwrap()).unwrap();
        assert_eq!(export["inventory"][0]["id"], pokemon.id.as_str());
        assert_eq!(
            export["inventory"][0]["referencePin"]["datasetId"],
            old_namespace
        );
        assert_eq!(
            export["inventory"][0]["referencePin"]["speciesId"],
            "testling"
        );
        assert!(export["tagDefinitions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tag| tag["name"] == "Unassigned personal tag"));
        let text = serde_json::to_string(&export).unwrap();
        assert!(!text.contains("baseStats"));
        assert!(!text.contains("sourceSha256"));
        assert!(export.get("screenshots").is_none());
        let target = path.with_extension("json");
        write_atomic(&target, &repo.structured_backup().unwrap()).unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&std::fs::read(&target).unwrap()).unwrap()["format"],
            "tcm-backup-1"
        );
        drop(repo);
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(target);
    }
}
