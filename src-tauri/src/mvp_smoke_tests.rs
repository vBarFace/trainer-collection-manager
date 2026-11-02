//! Native vertical smoke test using original fictional source data and user
//! observations. This exercises application contracts, not real game accuracy.
use crate::individual_details::IndividualDetails;
use crate::inventory::{AppraisalIvs, InventoryRepository, PokemonInput, PowerUpConfirmation};
use crate::reference::{Identity, ResolutionState};
use crate::reference_store::{digest, LocalPackage};
use serde_json::{json, Value};
use std::path::PathBuf;

struct TemporaryDatabase(PathBuf);

impl TemporaryDatabase {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "tcm-mvp-smoke-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )))
    }
}

impl Drop for TemporaryDatabase {
    fn drop(&mut self) {
        // Delete only files owned by this test; never recurse or touch originals.
        let _ = std::fs::remove_file(&self.0);
        for suffix in ["-wal", "-shm"] {
            let mut sidecar = self.0.as_os_str().to_os_string();
            sidecar.push(suffix);
            let _ = std::fs::remove_file(PathBuf::from(sidecar));
        }
    }
}

fn assert_no_private_image_or_reference_payload(value: &Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                assert!(
                    ![
                        "png",
                        "filename",
                        "fileName",
                        "path",
                        "screenshots",
                        "catalog",
                        "cpMultipliers",
                        "sourceSha256",
                        "artifacts",
                        "baseStats"
                    ]
                    .contains(&key.as_str()),
                    "Unexpected image or downloaded reference payload in backup: {key}"
                );
                assert_no_private_image_or_reference_payload(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                assert_no_private_image_or_reference_payload(value);
            }
        }
        _ => {}
    }
}

#[test]
fn offline_reference_import_collection_roadmap_confirmation_restart_and_backup() {
    let database = TemporaryDatabase::new();
    let mut repository = InventoryRepository::open(&database.0).unwrap();
    let source = json!({
        "pokemon_stats": [
            {"pokemon_id":60000,"pokemon_name":"Fictionling","form":"Normal","base_attack":100,"base_defense":100,"base_stamina":100},
            {"pokemon_id":60001,"pokemon_name":"Fictionwing","form":"Normal","base_attack":120,"base_defense":120,"base_stamina":120}
        ],
        "pokemon_types": [
            {"pokemon_id":60000,"form":"Normal","type":["Normal"]},
            {"pokemon_id":60001,"form":"Normal","type":["Normal","Flying"]}
        ],
        // Original synthetic multiplier grid. These values assert no game fact.
        "cp_multiplier": (0..79).map(|index|json!({
            "level":1.0+f64::from(index)/2.0,
            "multiplier":0.1+f64::from(index)*0.005
        })).collect::<Vec<_>>()
    });
    let raw = serde_json::to_vec(&source).unwrap();
    let normalized = crate::source_adapter::normalize(
        "pogoapi",
        &raw,
        "original-fictional-smoke-1",
        "2026-11-01",
    )
    .unwrap();
    let namespace = normalized.catalog["dataset"]["datasetId"]
        .as_str()
        .unwrap()
        .to_owned();
    repository
        .install_reference(LocalPackage {
            catalog: normalized.catalog,
            cp_multipliers: normalized.cp_multipliers,
            combat: normalized.combat,
            preparation: normalized.preparation,
            trainer_combat: normalized.trainer_combat,
            coverage: normalized.coverage,
            limitations: normalized.limitations,
            source_sha256: digest(&raw),
            provider: "pogoapi".into(),
            artifacts: vec![],
        })
        .unwrap();

    let input = PokemonInput {
        display_name: "My fictional collection individual".into(),
        cp: None,
        ivs: Some(AppraisalIvs {
            attack: 0,
            defense: 15,
            hp: 15,
        }),
    };
    let first = repository
        .confirm_batch_import("00000000-0000-4000-8000-000000000031", &input, &[])
        .unwrap()
        .0;
    let retry = repository
        .confirm_batch_import("00000000-0000-4000-8000-000000000031", &input, &[])
        .unwrap()
        .0;
    assert_eq!(retry.id, first.id);
    let second = repository
        .confirm_batch_import("00000000-0000-4000-8000-000000000032", &input, &[])
        .unwrap()
        .0;
    assert_ne!(second.id, first.id);
    for individual in [&first, &second] {
        repository
            .correct_identity(
                &individual.id,
                None,
                Some(Identity {
                    species: "Fictionling".into(),
                    form: Some("Normal".into()),
                }),
            )
            .unwrap();
    }
    let revision = repository
        .get(&first.id)
        .unwrap()
        .strategic_observation_revision;
    repository
        .save_individual_details(
            &first.id,
            revision,
            &repository
                .individual_details(&first.id)
                .unwrap()
                .evidence_token,
            IndividualDetails {
                fast_move: Some("Fictional quick move".into()),
                charged_move: Some("Fictional charged move".into()),
                level: Some(10.0),
                level_evidence: Some("Explicit fictional manually confirmed evidence".into()),
                favorite: Some(true),
                protected: true,
                ..Default::default()
            },
        )
        .unwrap();
    repository
        .save_strategy_preferences(0, "great", Some(0))
        .unwrap();
    let assessment = repository.collection_strategy().unwrap();
    assert_eq!(assessment.reports.len(), 2);
    let report = assessment
        .reports
        .iter()
        .find(|report| report.pokemon_id == first.id.as_str())
        .unwrap();
    assert!(report.protected);
    assert_eq!(report.unique_form, Some(false));
    assert_eq!(report.duplicate_count, 1);
    assert_eq!(report.duplicate_ids, vec![second.id.as_str()]);
    assert!(report.leagues.iter().all(|league| {
        league
            .potential
            .as_ref()
            .is_some_and(|potential| potential.level_ceiling == 40.0)
    }));
    assert!(report.recommendations.iter().all(|recommendation| {
        !recommendation.kind.contains("TRANSFER") && recommendation.kind != "EVOLVE"
    }));
    assert!(repository.tags_for(&first.id).unwrap().is_empty());

    let roadmap = repository
        .generate_roadmap(&first.id, &assessment.context)
        .unwrap();
    let postponed = repository
        .update_roadmap_progress(
            &first.id,
            &assessment.context,
            roadmap.revision.parse().unwrap(),
            "wait-investment",
            "postponed",
        )
        .unwrap();
    assert_eq!(postponed.progress["wait-investment"], "postponed");
    assert!(repository
        .update_roadmap_progress(
            &first.id,
            &assessment.context,
            roadmap.revision.parse().unwrap(),
            "wait-investment",
            "ignored",
        )
        .is_err());
    let before_action = repository.get(&first.id).unwrap();
    repository
        .confirm_power_up(
            &first.id,
            PowerUpConfirmation {
                request_id: "00000000-0000-4000-8000-000000000033",
                expected_revision: before_action.strategic_observation_revision,
                cp: None,
                performed_in_game_confirmed: true,
            },
        )
        .unwrap();
    let after_action = repository.get(&first.id).unwrap();
    assert_eq!(after_action.id, first.id);
    assert_eq!(after_action.cp, before_action.cp);
    assert_eq!(after_action.ivs, before_action.ivs);
    assert_eq!(
        after_action.strategic_observation_revision,
        before_action.strategic_observation_revision
    );
    assert!(repository.roadmap(&first.id).unwrap().stale);
    let new_context = repository.collection_strategy().unwrap().context;
    assert_ne!(new_context, assessment.context);
    let regenerated = repository
        .generate_roadmap(&first.id, &new_context)
        .unwrap();
    assert!(!regenerated.stale);
    assert!(regenerated.progress.is_empty());
    assert_eq!(
        regenerated.previous.as_ref().unwrap().progress["wait-investment"],
        "postponed"
    );
    assert_eq!(
        regenerated.previous.as_ref().unwrap().context,
        assessment.context
    );
    drop(repository);

    let reopened = InventoryRepository::open(&database.0).unwrap();
    assert_eq!(reopened.list().unwrap().len(), 2);
    assert_eq!(
        reopened.get(&first.id).unwrap().reference.state,
        ResolutionState::Resolved
    );
    assert_eq!(reopened.history(&first.id).unwrap().len(), 1);
    assert_eq!(
        reopened.history(&first.id).unwrap()[0].request_id,
        "00000000-0000-4000-8000-000000000033"
    );
    let saved = reopened.roadmap(&first.id).unwrap();
    assert!(!saved.stale);
    assert_eq!(saved.current_context, new_context);
    assert_eq!(saved.report.unwrap().pokemon_id, first.id.as_str());
    assert_eq!(reopened.strategy_preferences().unwrap().goal, "great");
    assert_eq!(
        reopened.strategy_preferences().unwrap().stardust_budget,
        Some(0)
    );
    let backup: Value = serde_json::from_slice(&reopened.structured_backup().unwrap()).unwrap();
    assert_eq!(backup["format"], "tcm-backup-1");
    let exported = backup["inventory"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"] == first.id.as_str())
        .unwrap();
    assert_eq!(exported["referencePin"]["datasetId"], namespace);
    assert_eq!(exported["referencePin"]["speciesId"], "dex-60000");
    assert_eq!(exported["referencePin"]["formId"], "dex-60000-normal");
    assert_eq!(exported["confirmedHistory"].as_array().unwrap().len(), 1);
    assert_eq!(backup["preferences"]["goal"], "great");
    assert_eq!(backup["roadmaps"].as_array().unwrap().len(), 1);
    assert_no_private_image_or_reference_payload(&backup);
    drop(reopened);
}
