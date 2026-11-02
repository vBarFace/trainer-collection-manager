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
        route_intent: None,
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
fn manual_route_survives_optional_priority_changes_and_unknown_target_cpm_blocks_power_up() {
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
        Some(selected.id.clone())
    );
    let plan = next.reports[0].preparation.as_ref().unwrap();
    assert_eq!(plan.planning_mode.as_deref(), Some("automatic_multi_role"));
    assert!(plan.paths.iter().any(|p| p.role == "ATTACKER"));
    assert!(plan.paths.iter().any(|p| p.role == "ROCKET"));
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
                && p.role == "ATTACKER"
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

#[test]
fn joint_route_retains_every_qualified_roles_unverified_conditions() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    let data = package();
    repo.install_reference(data.clone()).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 29.5);
    repo.save_preparation_resources(&id, 0, budget()).unwrap();
    let pokemon = repo.get(&id).unwrap();
    let detail = repo.individual_details(&id).unwrap();
    let gym = crate::gym::GymModel::new(&data).unwrap();
    let roles = crate::roles::RoleModel::new(Some(&data));
    let mut plan = crate::preparation::plan(
        &pokemon,
        &detail,
        Some(&data),
        gym.as_ref(),
        repo.preparation_resources(&id).unwrap(),
        None,
        false,
        "gym_attack",
        &roles,
        &[],
    );
    let attack = plan
        .paths
        .iter()
        .find(|p| p.target_form_id == "dex-61001-normal" && p.target_level == Some(30.0))
        .unwrap()
        .clone();
    let mut rocket = attack.clone();
    rocket.id = "test-rocket-route".into();
    rocket.role = "ROCKET".into();
    rocket.served_roles = vec!["ROCKET".into()];
    rocket.executable = false;
    rocket.blockers = vec!["Rocket lineup and shields are unverified".into()];
    plan.paths = vec![attack, rocket];
    let decision = crate::decision::IndividualDecision {
        summary: "Fictional supported uses".into(),
        missing: vec![],
        suggestions: ["ATTACKER", "ROCKET"]
            .iter()
            .map(|tag| crate::decision::SuggestedUse {
                id: tag.to_string(),
                tag: tag.to_string(),
                state: "preparation_potential".into(),
                target_form_id: Some("dex-61001-normal".into()),
                target_name: Some("Sampleton".into()),
                explanation: "Original fictional role evidence".into(),
                evidence: vec![],
                missing: vec![],
            })
            .collect(),
    };
    crate::integrated_planning::finalize(&mut plan, &decision, &pokemon, &detail, Some(&data), &[]);
    assert_eq!(plan.paths.len(), 1);
    assert_eq!(plan.paths[0].served_roles, vec!["ATTACKER", "ROCKET"]);
    assert!(!plan.paths[0].executable);
    assert!(plan.paths[0].blockers.iter().any(|b| b.contains("shields")));
}

#[test]
fn automatic_routes_review_current_form_league_options_before_evolving() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    repo.install_reference(package()).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 29.5);
    let league = repo
        .tags()
        .unwrap()
        .into_iter()
        .find(|t| t.name == "PVP 1500")
        .unwrap();
    repo.assign_tag(&id, league.id).unwrap();
    let result = repo.individual_decision(&id).unwrap();
    let plan = result.report.preparation.unwrap();
    let evolving = plan
        .paths
        .iter()
        .filter(|p| p.actions.iter().any(|a| a.kind == "evolve"))
        .collect::<Vec<_>>();
    assert!(!evolving.is_empty());
    for path in evolving {
        assert!(!path.executable);
        assert!(plan
            .conflicts
            .iter()
            .any(|c| c.path_id == path.id && c.role == "PVP 1500" && c.severity == "review"));
        assert!(
            path.projected_cp.is_some_and(|cp| cp <= 1500),
            "A within-cap evolution needs review, not a fabricated CP-limit violation"
        );
    }
    assert_eq!(
        repo.get(&id).unwrap().reference.species.unwrap().name,
        "Sampleton"
    );
}

#[test]
fn explicit_league_tag_blocks_evolution_routes_that_exceed_its_projected_cp_cap() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    let mut data = package();
    data.catalog["forms"][1]["baseStats"]["attack"] = serde_json::json!(1000);
    repo.install_reference(data).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 29.5);
    let tag = repo
        .tags()
        .unwrap()
        .into_iter()
        .find(|t| t.name == "PVP 1500")
        .unwrap();
    repo.assign_tag(&id, tag.id).unwrap();
    let reviewed = repo.individual_decision(&id).unwrap();
    let plan = reviewed.report.preparation.unwrap();
    let routes = plan
        .paths
        .iter()
        .filter(|p| p.actions.iter().any(|a| a.kind == "evolve"))
        .collect::<Vec<_>>();
    assert!(!routes.is_empty());
    for route in routes {
        assert!(route.projected_cp.is_some_and(|cp| cp > 1500));
        assert!(!route.executable);
        assert!(plan
            .conflicts
            .iter()
            .any(|c| c.path_id == route.id && c.role == "PVP 1500" && c.severity == "blocked"));
    }
}

#[test]
fn unknown_level_does_not_propose_same_form_power_up_below_observed_cp() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    repo.install_reference(package()).unwrap();
    let pokemon = repo
        .create(&PokemonInput {
            display_name: "Sampleton".into(),
            cp: Some(330),
            ivs: Some(AppraisalIvs {
                attack: 15,
                defense: 15,
                hp: 15,
            }),
        })
        .unwrap();
    repo.correct_identity(
        &pokemon.id,
        None,
        Some(Identity {
            species: "Sampleton".into(),
            form: Some("Normal".into()),
        }),
    )
    .unwrap();
    let result = repo.individual_decision(&pokemon.id).unwrap();
    let plan = result.report.preparation.unwrap();
    assert_eq!(plan.current_level, None);
    assert!(plan
        .paths
        .iter()
        .filter(|p| p.target_form_id == "dex-61001-normal"
            && p.actions.iter().any(|a| a.kind == "power_up"))
        .all(|p| p.projected_cp.is_none_or(|cp| cp >= 330)));
    assert!(plan
        .missing
        .iter()
        .any(|m| m.contains("lower projected CP")));
}

#[test]
fn automatic_model_checkpoint_uses_a_calculated_cp_target_with_an_explicit_policy() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    repo.install_reference(package()).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 20.0);
    let result = repo.individual_decision(&id).unwrap();
    let plan = result.report.preparation.unwrap();
    let path = plan
        .paths
        .iter()
        .find(|p| {
            p.target_form_id == "dex-61001-normal"
                && p.target_kind.as_deref() == Some("model_checkpoint")
                && p.role == "ATTACKER"
        })
        .unwrap();
    // Fictional m=.1+level/100, attack115, Defense215,m_def=.4:
    // L39 yields (5*4+40)/7 versus L40's (5*5+41)/7, first >=90%.
    assert_eq!(path.target_level, Some(39.0));
    assert_eq!(path.projected_cp, Some(317));
    assert!(path.benefit.contains("90%"));
    assert!(path.benefit.contains("not a battle breakpoint"));
}

fn broad_fictional_reference() -> LocalPackage {
    let mut data = package();
    for index in 0..10 {
        let dex = 61200 + index;
        let form_id = format!("dex-{dex}-normal");
        let mut species = data.catalog["species"][0].clone();
        species["id"] = format!("dex-{dex}").into();
        species["dexNumber"] = serde_json::json!(dex);
        species["name"] = format!("Guardling{index}").into();
        species["aliases"] = serde_json::json!([]);
        data.catalog["species"]
            .as_array_mut()
            .unwrap()
            .push(species);
        let mut form = data.catalog["forms"][0].clone();
        form["id"] = form_id.clone().into();
        form["speciesId"] = format!("dex-{dex}").into();
        form["baseStats"] = serde_json::json!({"attack":10,"defense":10,"stamina":10});
        form["evolutions"] = serde_json::Value::Null;
        data.catalog["forms"].as_array_mut().unwrap().push(form);
        let mut pool = data.combat.as_ref().unwrap().pools[0].clone();
        pool.form_id = form_id;
        data.combat.as_mut().unwrap().pools.push(pool);
    }
    data.validate().unwrap();
    data
}

#[test]
fn special_preservation_overrides_automatic_investment_without_erasing_qualified_uses() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    repo.install_reference(broad_fictional_reference()).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 20.0);
    repo.save_preparation_resources(&id, 0, budget()).unwrap();
    let tag = repo
        .tags()
        .unwrap()
        .into_iter()
        .find(|t| t.name == "SAVE/SPECIAL")
        .unwrap();
    repo.assign_tag(&id, tag.id).unwrap();
    let result = repo.individual_decision(&id).unwrap();
    assert!(result
        .report
        .decision
        .as_ref()
        .unwrap()
        .suggestions
        .iter()
        .any(|s| s.tag == "ATTACKER" && s.state == "preparation_potential"));
    let plan = result.report.preparation.as_ref().unwrap();
    let primary = plan
        .paths
        .iter()
        .find(|p| Some(&p.id) == plan.primary_path_id.as_ref())
        .unwrap();
    assert!(primary.actions.iter().all(|a| a.kind == "use_now"));
    assert_eq!(
        (
            primary.cost.candy,
            primary.cost.candy_xl,
            primary.cost.stardust
        ),
        (Some(0), Some(0), Some(0))
    );
    assert!(!result
        .report
        .recommendations
        .iter()
        .any(|r| r.id == "preparation-power_up" || r.id == "preparation-evolve"));
    assert!(repo
        .tags_for(&id)
        .unwrap()
        .iter()
        .any(|t| t.name == "SAVE/SPECIAL"));
}

#[test]
fn observed_elite_move_is_not_automatically_rerolled_for_an_ordinary_set() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    repo.install_reference(broad_fictional_reference()).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 20.0);
    repo.save_preparation_resources(&id, 0, budget()).unwrap();
    let details = repo.individual_details(&id).unwrap();
    let mut values = details.values;
    values.charged_move = Some("Historic Sample".into());
    repo.save_individual_details(
        &id,
        details.observation_revision.parse().unwrap(),
        &details.evidence_token,
        values,
    )
    .unwrap();
    let result = repo.individual_decision(&id).unwrap();
    let plan = result.report.preparation.as_ref().unwrap();
    assert!(plan.conflicts.iter().any(|c| c.role == "Move preservation"));
    let primary = plan
        .primary_path_id
        .as_ref()
        .and_then(|id| plan.paths.iter().find(|p| p.id == *id))
        .unwrap();
    assert!(!primary
        .actions
        .iter()
        .any(|a| a.kind == "change_moves" || a.kind == "evolve"));
    assert_eq!(
        repo.individual_details(&id)
            .unwrap()
            .values
            .charged_move
            .as_deref(),
        Some("Historic Sample")
    );
}

#[test]
fn integrated_route_continues_through_two_confirmed_evolutions_on_the_same_individual() {
    let db = Database::new();
    let mut repo = InventoryRepository::open(&db.0).unwrap();
    let mut data = broad_fictional_reference();
    for form in data.catalog["forms"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .filter(|f| f["id"].as_str().is_some_and(|id| id.starts_with("dex-612")))
    {
        form["baseStats"] = serde_json::json!({"attack":150,"defense":150,"stamina":150});
    }
    data.catalog["forms"][1]["baseStats"]["attack"] = serde_json::json!(120);
    data.catalog["species"].as_array_mut().unwrap().push(
        serde_json::json!({"id":"dex-61004","dexNumber":61004,"name":"Samplecrown","aliases":[]}),
    );
    data.catalog["forms"].as_array_mut().unwrap().push(serde_json::json!({"id":"dex-61004-normal","speciesId":"dex-61004","name":"Normal","aliases":[],"types":["normal"],"baseStats":{"attack":300,"defense":150,"stamina":150},"evolutions":null}));
    data.catalog["forms"][1]["evolutions"] = serde_json::json!([{"targetFormId":"dex-61004-normal","requirements":["50 fictional Candy; check in game"]}]);
    let mut pool = data.combat.as_ref().unwrap().pools[0].clone();
    pool.form_id = "dex-61004-normal".into();
    data.combat.as_mut().unwrap().pools.push(pool);
    let reference = data.preparation.as_mut().unwrap();
    reference
        .evolutions
        .push(crate::preparation_reference::EvolutionBranch {
            from_form_id: "dex-61002-normal".into(),
            to_form_id: "dex-61004-normal".into(),
            candy: 50,
            requirements: vec![],
            requirements_complete: false,
            free_if_traded: false,
        });
    let mut access = reference.move_access[0].clone();
    access.form_id = "dex-61004-normal".into();
    reference.move_access.push(access);
    data.validate().unwrap();
    repo.install_reference(data).unwrap();
    let id = specimen(&repo);
    level(&repo, &id, 20.0);
    repo.save_preparation_resources(&id, 0, budget()).unwrap();
    let applied = repo
        .tags()
        .unwrap()
        .into_iter()
        .find(|t| t.name == "ATTACKER")
        .unwrap();
    repo.assign_tag(&id, applied.id).unwrap();
    let tags = repo.tags_for(&id).unwrap();
    let ivs = repo.get(&id).unwrap().ivs;
    let initial = repo.individual_decision(&id).unwrap();
    let suggestions = &initial.report.decision.as_ref().unwrap().suggestions;
    for tag in ["ATTACKER", "GYM DEF"] {
        assert!(suggestions.iter().any(|s| s.tag == tag
            && s.target_form_id.as_deref() == Some("dex-61004-normal")
            && s.state == "preparation_potential"));
    }
    let plan = initial.report.preparation.as_ref().unwrap();
    assert!(plan.paths.iter().all(|p|p.future_roles.is_empty()),"Calibrated future evidence is stored once in the integrated decision, not repeated legacy reports");
    assert!(
        suggestions
            .iter()
            .find(|s| s.tag == "GYM DEF" && s.target_form_id.as_deref() == Some("dex-61004-normal"))
            .unwrap()
            .evidence
            .iter()
            .any(|e| e.starts_with("Typing")),
        "Future defender typing guidance remains in its authoritative suggestion"
    );
    let proposed = plan
        .paths
        .iter()
        .find(|p| Some(&p.id) == plan.primary_path_id.as_ref())
        .unwrap();
    assert_eq!(proposed.target_form_id, "dex-61004-normal");
    assert_eq!(
        proposed
            .actions
            .iter()
            .filter(|a| a.kind == "evolve")
            .count(),
        2
    );
    assert_ne!(
        initial
            .report
            .recommendations
            .iter()
            .find(|r| r.id == "gym-attack-potential")
            .unwrap()
            .kind,
        "ATTACKER",
        "Future qualification must not approve the weak current form"
    );
    repo.select_preparation_path(&id, &initial.context, &proposed.id)
        .unwrap();
    let chosen = repo.individual_decision(&id).unwrap();
    let intent = repo
        .preparation_resources(&id)
        .unwrap()
        .values
        .route_intent
        .unwrap();
    assert_eq!(intent.target_form_id, "dex-61004-normal");
    // Editing an allocation must not silently clear the accepted destination.
    let budget_now = repo.preparation_resources(&id).unwrap();
    repo.save_preparation_resources(&id, budget_now.revision.parse().unwrap(), budget())
        .unwrap();
    assert_eq!(
        repo.preparation_resources(&id).unwrap().values.route_intent,
        Some(intent)
    );
    for (target, request_id, remaining) in [
        (
            "dex-61002-normal",
            "20000000-0000-4000-8000-000000000061",
            1,
        ),
        (
            "dex-61004-normal",
            "20000000-0000-4000-8000-000000000062",
            0,
        ),
    ] {
        let options = repo.evolution_options(&id).unwrap();
        repo.confirm_evolution(
            &id,
            crate::inventory::EvolutionConfirmation {
                request_id,
                expected_revision: options
                    .expected_strategic_observation_revision
                    .parse()
                    .unwrap(),
                dataset_id: &options.dataset_id,
                dataset_version: &options.dataset_version,
                target_form_id: target,
            },
        )
        .unwrap();
        let next = repo.individual_decision(&id).unwrap();
        assert_eq!(next.report.pokemon_id, id.as_str());
        assert_eq!(repo.get(&id).unwrap().cp, None);
        assert_eq!(repo.get(&id).unwrap().ivs, ivs);
        assert_eq!(repo.tags_for(&id).unwrap(), tags);
        let plan = next.report.preparation.as_ref().unwrap();
        assert_eq!(plan.current_level, None);
        let route = plan
            .paths
            .iter()
            .find(|p| Some(&p.id) == plan.selected_path_id.as_ref())
            .unwrap();
        assert_eq!(route.target_form_id, "dex-61004-normal");
        assert_eq!(
            route.actions.iter().filter(|a| a.kind == "evolve").count(),
            remaining
        );
        assert!(!next.roadmap.stale);
    }
    let latest = repo.individual_decision(&id).unwrap();
    assert_ne!(latest.context, chosen.context);
    assert!(repo.clear_preparation_route(&id, "stale context").is_err());
    repo.clear_preparation_route(&id, &latest.context).unwrap();
    let automatic = repo.individual_decision(&id).unwrap();
    assert!(automatic
        .report
        .preparation
        .as_ref()
        .unwrap()
        .selected_path_id
        .is_none());
    assert!(repo
        .preparation_resources(&id)
        .unwrap()
        .values
        .route_intent
        .is_none());
    assert_eq!(repo.tags_for(&id).unwrap(), tags);
}
