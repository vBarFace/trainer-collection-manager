//! Original fictional end-to-end gym reference, recommendation and roadmap cases.
use crate::gym::fictional_package;
use crate::individual_details::IndividualDetails;
use crate::inventory::{
    AppraisalIvs, InventoryRepository, PokemonId, PokemonInput, PowerUpConfirmation,
};
use crate::reference::Identity;

struct Database(std::path::PathBuf);
impl Database {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        Self(std::env::temp_dir().join(format!(
            "gym-flow-{}-{}-{}.sqlite3",
                std::process::id(),
            NEXT.fetch_add(1,std::sync::atomic::Ordering::Relaxed),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )))
    }
}
impl Drop for Database {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
        for suffix in ["-wal", "-shm"] {
            let mut path = self.0.as_os_str().to_os_string();
            path.push(suffix);
            let _ = std::fs::remove_file(std::path::PathBuf::from(path));
        }
    }
}
fn create(repo: &InventoryRepository, name: &str) -> PokemonId {
    let individual = repo
        .create(&PokemonInput {
            display_name: name.into(),
            cp: None,
            ivs: Some(AppraisalIvs {
                attack: 15,
                defense: 15,
                hp: 15,
            }),
        })
        .unwrap();
    repo.correct_identity(
        &individual.id,
        None,
        Some(Identity {
            species: name.into(),
            form: Some("Normal".into()),
        }),
    )
    .unwrap();
    individual.id
}

#[test]
fn external_potential_survives_collection_changes_and_gym_roadmap_preserves_facts() {
    let path = Database::new();
    let mut repo = InventoryRepository::open(&path.0).unwrap();
    repo.install_reference(fictional_package()).unwrap();
    let id = create(&repo, "Sampleton");
    repo.save_strategy_preferences(0, "gym_attack", None)
        .unwrap();
    let first = repo.collection_strategy().unwrap();
    let original = first.reports[0].gym.clone().unwrap();
    assert!(original.builds[0]
        .reference_best
        .contains(&"Sampleprime".into()));
    assert!(original.owned_alternative_pokemon_id.is_none());
    assert!(original.builds[0].reference_percentile < 100.0);
    let plan = repo.generate_roadmap(&id, &first.context).unwrap();
    assert!(plan
        .report
        .as_ref()
        .unwrap()
        .steps
        .iter()
        .any(|step| step.id == "gym-compare-reference" && step.rationale.contains("fire")));
    let revision = plan.revision.parse().unwrap();
    let observed = repo.get(&id).unwrap();
    assert!(repo
        .update_roadmap_progress(
            &id,
            &first.context,
            revision,
            "gym-review-moves",
            "completed"
        )
        .is_err());
    let requested = repo
        .update_roadmap_progress(
            &id,
            &first.context,
            revision,
            "gym-review-moves",
            "review_requested",
        )
        .unwrap();
    assert_eq!(requested.progress["gym-review-moves"], "review_requested");
    assert_eq!(repo.get(&id).unwrap(), observed);
    let other = create(&repo, "Sampleprime");
    let second = repo.collection_strategy().unwrap();
    let report = second
        .reports
        .iter()
        .find(|report| report.pokemon_id == id.as_str())
        .unwrap();
    let personalized = report.gym.as_ref().unwrap();
    assert_eq!(personalized.builds, original.builds);
    assert_eq!(
        personalized.owned_alternative_pokemon_id.as_deref(),
        Some(other.as_str())
    );
    assert_eq!(
        report
            .recommendations
            .iter()
            .find(|item| item.id == "gym-attack-potential")
            .unwrap()
            .priority,
        26
    );
    assert!(repo.roadmap(&id).unwrap().stale);
    assert!(repo
        .update_roadmap_progress(
            &id,
            &first.context,
            revision,
            "gym-compare-reference",
            "completed"
        )
        .is_err());
    let draft = repo.individual_details(&id).unwrap();
    repo.save_individual_details(
        &id,
        observed.strategic_observation_revision,
        &draft.evidence_token,
        IndividualDetails {
            fast_move: Some("Sample Tap".into()),
            charged_move: Some("Sample Slam".into()),
            protected: true,
            ..Default::default()
        },
    )
    .unwrap();
    let confirmed = repo.collection_strategy().unwrap();
    let report = confirmed
        .reports
        .iter()
        .find(|report| report.pokemon_id == id.as_str())
        .unwrap();
    assert_eq!(
        report.gym.as_ref().unwrap().builds[0].move_state,
        "confirmed"
    );
    assert!(report.protected);
    assert!(report
        .recommendations
        .iter()
        .all(|item| !item.kind.contains("TRANSFER")));
    assert!(repo.tags_for(&id).unwrap().is_empty());
    assert_eq!(repo.individual_details(&id).unwrap().values.level, None);
    assert_eq!(repo.get(&id).unwrap().cp, None);
    let new_plan = repo.generate_roadmap(&id, &confirmed.context).unwrap();
    assert!(new_plan.previous.is_some());
    let pokemon = repo.get(&id).unwrap();
    repo.confirm_power_up(
        &id,
        PowerUpConfirmation {
            request_id: "00000000-0000-4000-8000-000000000079",
            expected_revision: pokemon.strategic_observation_revision,
            cp: None,
            performed_in_game_confirmed: true,
        },
    )
    .unwrap();
    assert!(repo.roadmap(&id).unwrap().stale);
    let after = repo.collection_strategy().unwrap();
    assert_eq!(
        after
            .reports
            .iter()
            .find(|report| report.pokemon_id == id.as_str())
            .unwrap()
            .gym
            .as_ref()
            .unwrap()
            .builds[0]
            .move_state,
        "hypothetical"
    );
    assert_eq!(repo.get(&id).unwrap().id, id);
    drop(repo);
}

#[test]
fn combat_activation_rollback_restart_and_old_report_compatibility() {
    let path = Database::new();
    let mut repo = InventoryRepository::open(&path.0).unwrap();
    let package = fictional_package();
    repo.install_reference(package.clone()).unwrap();
    let id = create(&repo, "Sampleton");
    let first = repo.collection_strategy().unwrap();
    let old_score = first.reports[0].gym.as_ref().unwrap().builds[0].neutral_cycle_dps;
    repo.generate_roadmap(&id, &first.context).unwrap();
    let mut old_json = serde_json::to_value(&first.reports[0]).unwrap();
    old_json.as_object_mut().unwrap().remove("gym");
    let old: crate::strategy::StrategyReport = serde_json::from_value(old_json).unwrap();
    assert!(old.gym.is_none());
    let mut old_package = serde_json::to_value(&package).unwrap();
    old_package.as_object_mut().unwrap().remove("combat");
    let old_package: crate::reference_store::LocalPackage =
        serde_json::from_value(old_package).unwrap();
    old_package.validate().unwrap();
    let mut invalid = package.clone();
    invalid.combat.as_mut().unwrap().pools[0].form_id = "dex-61999-normal".into();
    assert!(repo.install_reference(invalid).is_err());
    assert_eq!(repo.collection_strategy().unwrap().context, first.context);
    let mut next = package.clone();
    let observer = InventoryRepository::open(&path.0).unwrap();
    next.catalog["dataset"]["version"] = serde_json::json!("original-gym-fixture-2");
    next.catalog["forms"][0]["baseStats"]["attack"] = serde_json::json!(300);
    next.source_sha256 = crate::reference_store::digest(b"original fictional changed combat input");
    next.combat.as_mut().unwrap().moves[0].power = 20.0;
    repo.install_reference(next).unwrap();
    assert!(repo.roadmap(&id).unwrap().stale);
    let second = repo.collection_strategy().unwrap();
    let concurrent = observer.collection_strategy().unwrap();
    assert_eq!(concurrent.context, second.context);
    assert_eq!(concurrent.reports[0].gym, second.reports[0].gym);
    assert_ne!(
        second.reports[0].gym.as_ref().unwrap().builds[0].neutral_cycle_dps,
        old_score
    );
    repo.generate_roadmap(&id, &second.context).unwrap();
    drop(observer);
    drop(repo);
    let mut reopened = InventoryRepository::open(&path.0).unwrap();
    assert!(!reopened.roadmap(&id).unwrap().stale);
    assert!(reopened.roadmap(&id).unwrap().report.unwrap().gym.is_some());
    reopened.rollback_reference().unwrap();
    assert!(reopened.roadmap(&id).unwrap().stale);
    assert_eq!(
        reopened.collection_strategy().unwrap().reports[0]
            .gym
            .as_ref()
            .unwrap()
            .builds[0]
            .neutral_cycle_dps,
        old_score
    );
    assert_eq!(reopened.get(&id).unwrap().id, id);
    drop(reopened);
}

#[test]
fn league_goal_and_stale_details_do_not_repurpose_the_gym_comparison_step() {
    let path = Database::new();
    let mut repo = InventoryRepository::open(&path.0).unwrap();
    repo.install_reference(fictional_package()).unwrap();
    let id = create(&repo, "Sampleton");
    let pokemon = repo.get(&id).unwrap();
    let draft = repo.individual_details(&id).unwrap();
    repo.save_individual_details(
        &id,
        pokemon.strategic_observation_revision,
        &draft.evidence_token,
        IndividualDetails {
            fast_move: Some("Sample Tap".into()),
            ..Default::default()
        },
    )
    .unwrap();
    repo.save_strategy_preferences(0, "master", None).unwrap();
    let first = repo.collection_strategy().unwrap();
    let steps = &first.reports[0].steps;
    assert!(steps
        .iter()
        .find(|step| step.id == "gym-compare-reference")
        .unwrap()
        .rationale
        .contains("normal attack coverage"));
    assert!(steps
        .iter()
        .find(|step| step.id == "compare-alternatives")
        .unwrap()
        .rationale
        .contains("Master League"));
    let individual = repo.get(&id).unwrap();
    repo.confirm_power_up(
        &id,
        PowerUpConfirmation {
            request_id: "00000000-0000-4000-8000-000000000080",
            expected_revision: individual.strategic_observation_revision,
            cp: None,
            performed_in_game_confirmed: true,
        },
    )
    .unwrap();
    let second = repo.collection_strategy().unwrap();
    let steps = &second.reports[0].steps;
    let gym = steps
        .iter()
        .find(|step| step.id == "gym-compare-reference")
        .unwrap();
    assert!(gym.rationale.contains("normal attack coverage"));
    assert!(gym.executable);
    let league = steps
        .iter()
        .find(|step| step.id == "compare-alternatives")
        .unwrap();
    assert!(!league.executable);
    assert!(league
        .prerequisites
        .contains(&"review-optional-observations".into()));
    drop(repo);
}
