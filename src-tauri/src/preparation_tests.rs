//! Original invented mechanics and specimens; no provider game records.
use crate::{
    gym::fictional_package,
    individual_details::IndividualDetails,
    inventory::{AppraisalIvs, InventoryRepository, PokemonId, PokemonInput, PowerUpConfirmation},
    preparation::{power_up_cost, ResourceCost},
    preparation_resources::ResourceValues,
    reference::Identity,
    reference_store::LocalPackage,
};

fn package() -> LocalPackage {
    let mut package = fictional_package();
    package.catalog["dataset"]["version"] = "preparation-fiction-1".into();
    package.catalog["forms"][0]["evolutions"] = serde_json::json!([{"targetFormId":"dex-61002-normal","requirements":["25 fictional Candy; verify in game"]}]);
    package.cp_multipliers = (2..=100)
        .map(|half| {
            let level = f64::from(half) / 2.0;
            (level, 0.1 + level / 100.0)
        })
        .collect();
    package.preparation=Some(serde_json::from_value(serde_json::json!({
        "version":crate::preparation_reference::VERSION,
        "powerUpCosts":(2..100).map(|half|{let from=f64::from(half)/2.0;serde_json::json!({"fromLevel":from,"toLevel":from+0.5,"candy":if from<40.0{1}else{0},"candyXl":if from>=40.0{2}else{0},"stardust":10})}).collect::<Vec<_>>(),
        "evolutions":[{"fromFormId":"dex-61001-normal","toFormId":"dex-61002-normal","candy":25,"requirements":[{"kind":"item","description":"Fiction stone"}],"requirementsComplete":false,"freeIfTraded":false}],
        "moveAccess":[{"formId":"dex-61001-normal","fastMoves":["Sample Tap"],"chargedMoves":["Sample Slam"],"eliteFastMoves":[],"eliteChargedMoves":["Historic Sample"]},{"formId":"dex-61002-normal","fastMoves":["Sample Tap"],"chargedMoves":["Sample Slam"],"eliteFastMoves":[],"eliteChargedMoves":[]}],
        "limitations":["Original invented preparation test mechanics"]
    })).unwrap());
    package.validate().unwrap();
    package
}
struct Database(std::path::PathBuf);
impl Database {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let name = format!(
            "prep-{}-{}-{}.sqlite3",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        Self(std::env::temp_dir().join(name))
    }
}
impl Drop for Database {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let mut p = self.0.as_os_str().to_os_string();
            p.push(suffix);
            let _ = std::fs::remove_file(std::path::PathBuf::from(p));
        }
    }
}
fn specimen(repo: &InventoryRepository) -> PokemonId {
    let p = repo
        .create(&PokemonInput {
            display_name: "Sampleton".into(),
            cp: None,
            ivs: Some(AppraisalIvs {
                attack: 15,
                defense: 15,
                hp: 15,
            }),
        })
        .unwrap();
    repo.correct_identity(
        &p.id,
        None,
        Some(Identity {
            species: "Sampleton".into(),
            form: Some("Normal".into()),
        }),
    )
    .unwrap();
    p.id
}
fn level(repo: &InventoryRepository, id: &PokemonId, level: f64) {
    let details = repo.individual_details(id).unwrap();
    repo.save_individual_details(
        id,
        details.observation_revision.parse().unwrap(),
        &details.evidence_token,
        IndividualDetails {
            level: Some(level),
            level_evidence: Some("Manual fictional level observation".into()),
            fast_move: Some("Sample Tap".into()),
            charged_move: Some("Sample Slam".into()),
            ..Default::default()
        },
    )
    .unwrap();
}
fn budget() -> ResourceValues {
    ResourceValues {
        candy: Some(1000),
        candy_xl: Some(1000),
        stardust: Some(100000),
        maximum_level: Some(50.0),
        normal_costs_confirmed: true,
        selected_path_id: None,
    }
}

#[test]
fn sums_only_explicit_half_level_intervals_and_xl_boundary() {
    let package = package();
    let reference = package.preparation.as_ref().unwrap();
    assert_eq!(power_up_cost(reference, 29.5, 30.0), Some((1, 0, 10)));
    assert_eq!(power_up_cost(reference, 39.5, 40.5), Some((1, 2, 20)));
    assert_eq!(power_up_cost(reference, 40.0, 50.0), Some((0, 40, 200)));
    assert_eq!(power_up_cost(reference, 40.0, 40.0), Some((0, 0, 0)));
    let mut missing = reference.clone();
    missing.power_up_costs.retain(|c| c.from_level != 40.0);
    assert!(power_up_cost(&missing, 39.5, 40.5).is_none());
    for (a, b) in [
        (30.0, 29.5),
        (1.25, 30.0),
        (0.0, 30.0),
        (30.0, 51.0),
        (f64::NAN, 40.0),
    ] {
        assert!(power_up_cost(reference, a, b).is_none());
    }
}

#[test]
fn evaluates_evolution_power_up_totals_current_benefit_and_retention_together() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    repo.install_reference(package()).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 29.5);
    repo.save_preparation_resources(&id, 0, budget()).unwrap();
    let collection = repo.collection_strategy().unwrap();
    let report = &collection.reports[0];
    let plan = report.preparation.as_ref().unwrap();
    assert_eq!(plan.current_level, Some(29.5));
    assert!(report
        .roles
        .iter()
        .any(|r| r.role == "transfer" && r.state != "transfer"));
    let direct = plan
        .paths
        .iter()
        .find(|p| p.target_form_id == "dex-61001-normal" && p.target_level == Some(30.0))
        .unwrap();
    assert_eq!(
        direct.cost,
        ResourceCost {
            candy: Some(1),
            candy_xl: Some(0),
            stardust: Some(10),
            items: vec![]
        }
    );
    // Independently hand-calculated neutral fictional cycle: (5*4+33)/7.
    assert!(direct.benefit.contains("7.57"));
    assert!(direct.benefit.contains("7.43"));
    assert!(direct.executable);
    let evolved = plan
        .paths
        .iter()
        .find(|p| p.target_form_id == "dex-61002-normal" && p.target_level == Some(30.0))
        .unwrap();
    assert_eq!(evolved.cost.candy, Some(26));
    assert_eq!(evolved.cost.stardust, Some(10));
    assert_eq!(evolved.actions[0].kind, "evolve");
    assert_eq!(evolved.actions[1].kind, "power_up");
    assert_eq!(evolved.actions[2].kind, "change_moves");
    assert!(evolved.blockers.iter().any(|s| s.contains("prerequisites")));
    assert!(!evolved.executable);
    assert_eq!(repo.get(&id).unwrap().cp, None);
}

#[test]
fn unknown_level_variant_budget_and_stale_observation_never_become_zero_cost() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    repo.install_reference(package()).unwrap();
    let id = specimen(&repo);
    let report = repo.collection_strategy().unwrap();
    let plan = report.reports[0].preparation.as_ref().unwrap();
    assert!(plan
        .paths
        .iter()
        .filter(|p| p.actions.iter().any(|a| a.kind == "power_up"))
        .all(|p| p.cost.stardust.is_none() && !p.executable));
    level(&repo, &id, 29.5);
    let mut values = budget();
    values.stardust = Some(0);
    repo.save_preparation_resources(&id, 0, values).unwrap();
    let report = repo.collection_strategy().unwrap();
    assert!(report.reports[0]
        .preparation
        .as_ref()
        .unwrap()
        .paths
        .iter()
        .filter(|p| p.actions.iter().any(|a| a.kind == "power_up"))
        .all(|p| p.affordability == "over_budget"));
    let before = repo.get(&id).unwrap();
    repo.confirm_power_up(
        &id,
        PowerUpConfirmation {
            request_id: "20000000-0000-4000-8000-000000000001",
            expected_revision: before.strategic_observation_revision,
            cp: None,
            performed_in_game_confirmed: true,
        },
    )
    .unwrap();
    let report = repo.collection_strategy().unwrap();
    assert_eq!(
        report.reports[0]
            .preparation
            .as_ref()
            .unwrap()
            .current_level,
        None
    );
}

#[test]
fn selected_path_survives_restart_and_actions_recalculate_without_fabricating_progress() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    repo.install_reference(package()).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 29.5);
    repo.save_preparation_resources(&id, 0, budget()).unwrap();
    let old = repo.collection_strategy().unwrap();
    let stored = repo.generate_roadmap(&id, &old.context).unwrap();
    let path = old.reports[0]
        .preparation
        .as_ref()
        .unwrap()
        .paths
        .iter()
        .find(|p| p.target_form_id == "dex-61002-normal" && p.target_level == Some(30.0))
        .unwrap()
        .id
        .clone();
    assert!(repo.select_preparation_path(&id, "stale", &path).is_err());
    assert!(repo
        .select_preparation_path(&id, &old.context, "nonexistent")
        .is_err());
    repo.select_preparation_path(&id, &old.context, &path)
        .unwrap();
    assert!(repo.roadmap(&id).unwrap().stale);
    assert!(repo.save_preparation_resources(&id, 0, budget()).is_err());
    drop(repo);
    let repo = InventoryRepository::open(&db.0).unwrap();
    let next = repo.collection_strategy().unwrap();
    assert_ne!(old.context, next.context);
    let new = repo.generate_roadmap(&id, &next.context).unwrap();
    assert_eq!(
        new.report
            .as_ref()
            .unwrap()
            .preparation
            .as_ref()
            .unwrap()
            .selected_path_id
            .as_deref(),
        Some(path.as_str())
    );
    let steps = new
        .report
        .as_ref()
        .unwrap()
        .steps
        .iter()
        .filter(|s| s.id.starts_with("selected-preparation"))
        .collect::<Vec<_>>();
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[1].prerequisites, vec![steps[0].id.clone()]);
    assert!(new.previous.is_some());
    assert_eq!(stored.progress, new.progress);
    assert!(repo
        .update_roadmap_progress(
            &id,
            &new.context,
            new.revision.parse().unwrap(),
            &steps[0].id,
            "completed"
        )
        .is_err());
    assert_eq!(
        repo.get(&id).unwrap().reference.species.unwrap().name,
        "Sampleton"
    );
    let export: serde_json::Value =
        serde_json::from_slice(&repo.structured_backup().unwrap()).unwrap();
    assert_eq!(
        export["inventory"][0]["preparationResources"]["selectedPathId"],
        path
    );
    repo.delete(&id).unwrap();
    let count: i64 = repo
        .connection
        .query_row(
            "SELECT count(*) FROM application_settings WHERE key LIKE 'preparation-resources:%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn path_selection_is_not_reused_for_another_role_and_unknown_target_cpm_blocks_power_up() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    let mut data = package();
    data.cp_multipliers.retain(|(l, _)| *l <= 40.0);
    repo.install_reference(data).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 39.5);
    repo.save_preparation_resources(&id, 0, budget()).unwrap();
    let first = repo.collection_strategy().unwrap();
    let plan = first.reports[0].preparation.as_ref().unwrap();
    let path = plan
        .paths
        .iter()
        .find(|p| p.target_form_id == "dex-61001-normal" && p.target_level == Some(50.0))
        .unwrap();
    assert_eq!(path.cost.candy_xl, Some(40));
    assert!(!path.executable);
    assert!(path.blockers.iter().any(|s| s.contains("CPM")));
    let selected = plan
        .paths
        .iter()
        .find(|p| p.target_form_id == "dex-61001-normal" && p.target_level == Some(40.0))
        .unwrap();
    repo.select_preparation_path(&id, &first.context, &selected.id)
        .unwrap();
    repo.save_strategy_preferences(0, "rocket", None).unwrap();
    let next = repo.collection_strategy().unwrap();
    assert_eq!(
        next.reports[0]
            .preparation
            .as_ref()
            .unwrap()
            .selected_path_id,
        None
    );
    assert!(next.reports[0]
        .preparation
        .as_ref()
        .unwrap()
        .paths
        .iter()
        .filter(|p| p.id != "keep-current-state")
        .all(|p| p.role == "ROCKET"));
}

#[test]
fn second_charged_slot_is_part_of_current_baseline_and_shadow_damage_is_withheld() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    let mut data = package();
    let combat = data.combat.as_mut().unwrap();
    let mut slow = combat
        .moves
        .iter()
        .find(|m| m.kind == "charged")
        .unwrap()
        .clone();
    slow.id = "move-900003".into();
    slow.name = "Sample Slow".into();
    slow.power = 20.0;
    combat.moves.push(slow);
    combat.pools[0].charged_move_ids.push("move-900003".into());
    data.preparation.as_mut().unwrap().move_access[0]
        .charged_moves
        .push("Sample Slow".into());
    repo.install_reference(data).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 30.0);
    repo.save_preparation_resources(&id, 0, budget()).unwrap();
    let details = repo.individual_details(&id).unwrap();
    let mut values = details.values;
    values.charged_move = Some("Sample Slow".into());
    values.second_charged_move = Some("Sample Slam".into());
    repo.save_individual_details(
        &id,
        details.observation_revision.parse().unwrap(),
        &details.evidence_token,
        values.clone(),
    )
    .unwrap();
    let report = repo.collection_strategy().unwrap();
    let plan = report.reports[0].preparation.as_ref().unwrap();
    let same = plan
        .paths
        .iter()
        .find(|p| {
            p.id != "keep-current-state"
                && p.target_form_id == "dex-61001-normal"
                && p.target_level == Some(30.0)
        })
        .unwrap();
    assert!(same.benefit.contains("change +0.0%"));
    assert!(!same.actions.iter().any(|a| a.kind == "change_moves"));
    let details = repo.individual_details(&id).unwrap();
    values.shadow = Some(true);
    repo.save_individual_details(
        &id,
        details.observation_revision.parse().unwrap(),
        &details.evidence_token,
        values,
    )
    .unwrap();
    let report = repo.collection_strategy().unwrap();
    assert!(report.reports[0]
        .preparation
        .as_ref()
        .unwrap()
        .paths
        .iter()
        .all(|p| !p.benefit.contains("Fixed-scenario projected cycle")
            && !p.benefit.contains("Current confirmed set")));
}

#[test]
fn known_special_species_cost_override_cannot_be_overruled_by_ordinary_checkbox() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    let mut data = package();
    // Reassign an invented specimen to the known unsupported override identity;
    // no real stats, cost table or other game records enter the fixture.
    data.catalog["species"][0]["dexNumber"] = serde_json::json!(890);
    repo.install_reference(data).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 39.5);
    repo.save_preparation_resources(&id, 0, budget()).unwrap();
    let report = repo.collection_strategy().unwrap();
    let plan = report.reports[0].preparation.as_ref().unwrap();
    assert!(plan
        .missing
        .iter()
        .any(|s| s.contains("special Power Up cost override")));
    assert!(plan
        .paths
        .iter()
        .filter(|p| p.actions.iter().any(|a| a.kind == "power_up"))
        .all(|p| p.cost.candy.is_none()
            && p.cost.candy_xl.is_none()
            && p.cost.stardust.is_none()
            && !p.executable));
}

#[test]
fn unchanged_collection_reassessment_is_identical_across_repeats_noops_and_restart() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    let mut reference = package();
    reference.trainer_combat = Some(crate::trainer_combat_reference::fictional_reference());
    repo.install_reference(reference).unwrap();
    let id = specimen(&repo);
    let other = specimen(&repo);
    level(&repo, &id, 29.5);
    level(&repo, &other, 30.0);
    repo.save_preparation_resources(&id, 0, budget()).unwrap();
    let snapshot = repo.collection_strategy().unwrap();
    let expected = serde_json::to_vec(&snapshot).unwrap();
    let road = repo.generate_roadmap(&id, &snapshot.context).unwrap();
    let road_expected = serde_json::to_vec(&road).unwrap();
    let observed = repo.get(&id).unwrap();
    let tags = repo.tags_for(&id).unwrap();
    for _ in 0..10 {
        assert_eq!(
            serde_json::to_vec(&repo.collection_strategy().unwrap()).unwrap(),
            expected
        );
        assert_eq!(
            serde_json::to_vec(&repo.generate_roadmap(&id, &snapshot.context).unwrap()).unwrap(),
            road_expected
        );
    }
    // Saving the same confirmed values is not a semantic change.
    let details = repo.individual_details(&id).unwrap();
    repo.save_individual_details(
        &id,
        details.observation_revision.parse().unwrap(),
        &details.evidence_token,
        details.values,
    )
    .unwrap();
    repo.save_preparation_resources(&id, 1, budget()).unwrap();
    assert_eq!(
        serde_json::to_vec(&repo.collection_strategy().unwrap()).unwrap(),
        expected
    );
    drop(repo);
    let repo = InventoryRepository::open(&db.0).unwrap();
    assert_eq!(
        serde_json::to_vec(&repo.collection_strategy().unwrap()).unwrap(),
        expected
    );
    assert_eq!(
        serde_json::to_vec(&repo.roadmap(&id).unwrap()).unwrap(),
        road_expected
    );
    // A real budget change may change the decision; rerunning that new state
    // must then be stable too. Neither assessment rewrites Pokémon facts/tags.
    let mut zero = budget();
    zero.stardust = Some(0);
    repo.save_preparation_resources(&id, 1, zero).unwrap();
    let changed = repo.collection_strategy().unwrap();
    assert_ne!(changed.context, snapshot.context);
    assert_ne!(
        serde_json::to_vec(&changed.reports).unwrap(),
        serde_json::to_vec(&snapshot.reports).unwrap()
    );
    let updated = serde_json::to_vec(&changed).unwrap();
    for _ in 0..10 {
        assert_eq!(
            serde_json::to_vec(&repo.collection_strategy().unwrap()).unwrap(),
            updated
        );
    }
    assert_eq!(repo.get(&id).unwrap(), observed);
    assert_eq!(repo.tags_for(&id).unwrap(), tags);
}

#[test]
fn adding_an_unrelated_weaker_individual_does_not_rewrite_existing_advice() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    repo.install_reference(package()).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 30.0);
    let before = repo.collection_strategy().unwrap();
    let expected = serde_json::to_vec(
        before
            .reports
            .iter()
            .find(|r| r.pokemon_id == id.as_str())
            .unwrap(),
    )
    .unwrap();
    let added = repo
        .create(&PokemonInput {
            display_name: "Samplelittle".into(),
            cp: None,
            ivs: Some(AppraisalIvs {
                attack: 15,
                defense: 15,
                hp: 15,
            }),
        })
        .unwrap();
    repo.correct_identity(
        &added.id,
        None,
        Some(Identity {
            species: "Samplelittle".into(),
            form: Some("Normal".into()),
        }),
    )
    .unwrap();
    let after = repo.collection_strategy().unwrap();
    assert_ne!(
        before.context, after.context,
        "The collection snapshot changed"
    );
    assert_eq!(expected,serde_json::to_vec(after.reports.iter().find(|r|r.pokemon_id==id.as_str()).unwrap()).unwrap(),"A changed collection fingerprint alone does not establish changed advice for every individual");
}
