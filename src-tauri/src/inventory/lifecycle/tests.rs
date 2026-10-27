use super::*;
use serde_json::json;

const FIRST: &str = "11111111-1111-4111-8111-111111111111";
const SECOND: &str = "22222222-2222-4222-8222-222222222222";
const THIRD: &str = "33333333-3333-4333-8333-333333333333";

fn power_up<'a>(
    pokemon: &Pokemon,
    request_id: &'a str,
    cp: Option<i64>,
) -> PowerUpConfirmation<'a> {
    PowerUpConfirmation {
        request_id,
        expected_revision: pokemon.strategic_observation_revision,
        cp,
        performed_in_game_confirmed: true,
    }
}

#[test]
fn confirmed_power_up_changes_only_cp_and_preserves_individual_metadata() {
    let repo = repository();
    let initial = saved(&repo);
    let (initial, tags) = repo.evaluate(&initial.id).unwrap();
    let other = saved(&repo);
    let (after, after_tags) = repo
        .confirm_power_up(&initial.id, power_up(&initial, FIRST, Some(i64::MAX)))
        .unwrap();
    let mut expected = initial.clone();
    expected.cp = Some(i64::MAX);
    expected.strategic_observation_revision += 1;
    expected.updated_at = after.updated_at.clone();
    assert_eq!(after, expected);
    assert_eq!(after_tags, tags);
    assert_eq!(repo.get(&other.id).unwrap(), other);
    assert!(repo.history(&other.id).unwrap().is_empty());
    assert_eq!(repo.list().unwrap().len(), 2);
    let event = repo.history(&initial.id).unwrap().remove(0);
    assert_eq!(event.kind, "power_up");
    assert_eq!(event.request_id, FIRST);
    assert_eq!(event.pokemon_id, initial.id.as_str());
    assert_eq!(event.before.cp.as_deref(), Some("9007199254740993"));
    assert_eq!(event.after.cp.as_deref(), Some("9223372036854775807"));
    assert_eq!(event.before.identity, event.after.identity);
    assert_eq!(event.before.ivs, event.after.ivs);
    assert!(matches!(event.cp_source, Some(ObservationSource::Manual)));
    assert!(event.ivs_source.is_none());
    assert!(event.same_individual_confirmed);
    assert!(!event.confirmed_at.is_empty());
    assert_eq!(event.dataset_version, initial.reference.dataset.version);
}

#[test]
fn power_up_without_new_cp_keeps_known_and_unknown_observations_and_rejects_replay() {
    for cp in [None, Some(123)] {
        let repo = repository();
        let initial = repo
            .create(&PokemonInput {
                display_name: "Saved".into(),
                cp,
                ivs: None,
            })
            .unwrap();
        let (after, _) = repo
            .confirm_power_up(&initial.id, power_up(&initial, FIRST, None))
            .unwrap();
        assert_eq!(after, initial);
        let event = repo.history(&initial.id).unwrap().remove(0);
        assert_eq!(event.before.cp, cp.map(|cp| cp.to_string()));
        assert_eq!(event.after.cp, event.before.cp);
        assert!(event.cp_source.is_none());
        assert!(matches!(
            repo.confirm_power_up(&initial.id, power_up(&initial, FIRST, None)),
            Err(Error::AlreadyConfirmed)
        ));
        assert_eq!(repo.history(&initial.id).unwrap().len(), 1);
        assert_eq!(repo.list().unwrap().len(), 1);
    }
}

#[test]
fn unconfirmed_invalid_stale_and_missing_power_ups_never_write() {
    let repo = repository();
    let initial = saved(&repo);
    let mut cancelled = power_up(&initial, FIRST, Some(140));
    cancelled.performed_in_game_confirmed = false;
    assert!(matches!(
        repo.confirm_power_up(&initial.id, cancelled),
        Err(Error::InvalidInput(_))
    ));
    for cp in [0, -1] {
        assert!(matches!(
            repo.confirm_power_up(&initial.id, power_up(&initial, FIRST, Some(cp))),
            Err(Error::InvalidInput(_))
        ));
    }
    assert!(matches!(
        repo.confirm_power_up(&initial.id, power_up(&initial, "invalid", None)),
        Err(Error::InvalidInput(_))
    ));
    let mut stale = power_up(&initial, FIRST, Some(140));
    stale.expected_revision -= 1;
    assert!(matches!(
        repo.confirm_power_up(&initial.id, stale),
        Err(Error::ObservationConflict)
    ));
    assert!(matches!(
        repo.confirm_power_up(
            &PokemonId::parse(SECOND).unwrap(),
            power_up(&initial, FIRST, None)
        ),
        Err(Error::NotFound(_))
    ));
    assert_eq!(repo.get(&initial.id).unwrap(), initial);
    assert!(repo.history(&initial.id).unwrap().is_empty());
    // A manual correction and revert still invalidate a pending Power Up.
    let changed = repo
        .correct_cp(
            &initial.id,
            initial.cp,
            initial.strategic_observation_revision,
            Some(100),
        )
        .unwrap()
        .0;
    repo.correct_cp(
        &initial.id,
        changed.cp,
        changed.strategic_observation_revision,
        initial.cp,
    )
    .unwrap();
    assert!(matches!(
        repo.confirm_power_up(&initial.id, power_up(&initial, FIRST, None)),
        Err(Error::ObservationConflict)
    ));
    assert!(repo.history(&initial.id).unwrap().is_empty());
}

#[test]
fn power_up_rolls_back_cp_revision_and_history_on_insert_failure_or_overflow() {
    let repo = repository();
    let initial = saved(&repo);
    repo.connection.execute_batch("CREATE TRIGGER fail_event BEFORE INSERT ON pokemon_lifecycle_events BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();
    assert!(repo
        .confirm_power_up(&initial.id, power_up(&initial, FIRST, Some(300)))
        .is_err());
    assert_eq!(repo.get(&initial.id).unwrap(), initial);
    assert!(repo.history(&initial.id).unwrap().is_empty());
    repo.connection
        .execute_batch("DROP TRIGGER fail_event;")
        .unwrap();
    repo.connection
        .execute(
            "UPDATE pokemon SET strategic_observation_revision = ?2 WHERE id = ?1",
            params![initial.id.as_str(), i64::MAX],
        )
        .unwrap();
    let exhausted = repo.get(&initial.id).unwrap();
    assert!(repo
        .confirm_power_up(&initial.id, power_up(&exhausted, FIRST, Some(300)))
        .is_err());
    assert_eq!(repo.get(&initial.id).unwrap(), exhausted);
    assert!(repo.history(&initial.id).unwrap().is_empty());
    // A failed transaction does not consume the request; history-only confirmation still works.
    repo.confirm_power_up(&initial.id, power_up(&exhausted, FIRST, None))
        .unwrap();
    assert_eq!(repo.history(&initial.id).unwrap().len(), 1);
}

#[test]
fn corrections_evolution_reobservation_and_power_up_share_consistent_history() {
    let repo = repository();
    let initial = saved(&repo);
    let corrected = repo
        .correct_cp(
            &initial.id,
            initial.cp,
            initial.strategic_observation_revision,
            Some(123),
        )
        .unwrap()
        .0;
    assert!(repo.history(&initial.id).unwrap().is_empty());
    let evolved = evolve(&repo, &corrected, FIRST, "charmeleon-normal")
        .unwrap()
        .0;
    let powered = repo
        .confirm_power_up(&initial.id, power_up(&evolved, SECOND, Some(456)))
        .unwrap()
        .0;
    assert!(matches!(
        repo.reobserve(&initial.id, observation(&powered, SECOND)),
        Err(Error::AlreadyConfirmed)
    ));
    let observed = repo
        .reobserve(&initial.id, observation(&powered, THIRD))
        .unwrap()
        .0;
    assert_eq!(observed.id, initial.id);
    let history = repo.history(&initial.id).unwrap();
    assert_eq!(
        history
            .iter()
            .map(|event| event.kind.as_str())
            .collect::<Vec<_>>(),
        vec!["reobservation", "power_up", "evolution"]
    );
    assert_eq!(history[1].before.cp, None);
    assert_eq!(history[1].after.cp.as_deref(), Some("456"));
    assert_eq!(history[0].before.cp, history[1].after.cp);
    assert_eq!(repo.list().unwrap().len(), 1);
}

fn repository() -> InventoryRepository {
    let mut connection = Connection::open_in_memory().unwrap();
    database::configure_connection(&connection).unwrap();
    database::apply_migrations(&mut connection).unwrap();
    let mut repo = InventoryRepository {
        connection,
        catalog: Catalog::bundled().unwrap(),
    };
    repo.activate_reference(&fixture(false)).unwrap();
    repo
}

// Hand-authored test-only identities and edges; no imported game stats or costs.
fn fixture(branch: bool) -> Vec<u8> {
    let mut document = crate::reference::tests::document();
    document["dataset"]["version"] = json!("lifecycle-test-1");
    document["species"] = json!([
        {"id":"charmander", "dexNumber":60010,"name":"Charmander","aliases":[]},
        {"id":"charmeleon", "dexNumber":60011,"name":"Charmeleon","aliases":[]},
        {"id":"charizard", "dexNumber":60012,"name":"Charizard","aliases":[]},
        {"id":"testbranch", "dexNumber":60013,"name":"Testbranch","aliases":[]}
    ]);
    let mut first_edges = vec![json!({"targetFormId":"charmeleon-normal","requirements":null})];
    if branch {
        first_edges.push(json!({"targetFormId":"testbranch-normal","requirements":null}));
    }
    document["forms"] = json!([
        {"id":"charmander-normal","speciesId":"charmander","name":"Normal","aliases":[],"types":null,"baseStats":null,"evolutions":first_edges},
        {"id":"charmeleon-normal","speciesId":"charmeleon","name":"Normal","aliases":[],"types":null,"baseStats":null,"evolutions":[{"targetFormId":"charizard-normal","requirements":null}]},
        {"id":"charizard-normal","speciesId":"charizard","name":"Normal","aliases":[],"types":null,"baseStats":null,"evolutions":[]},
        {"id":"testbranch-normal","speciesId":"testbranch","name":"Normal","aliases":[],"types":null,"baseStats":null,"evolutions":null}
    ]);
    serde_json::to_vec(&document).unwrap()
}

fn saved(repo: &InventoryRepository) -> Pokemon {
    let pokemon = repo
        .create(&PokemonInput {
            display_name: "My nickname".into(),
            cp: Some(9007199254740993),
            ivs: Some(AppraisalIvs {
                attack: 15,
                defense: 14,
                hp: 13,
            }),
        })
        .unwrap();
    repo.assign_tag(&pokemon.id, repo.tags().unwrap()[0].id)
        .unwrap();
    repo.correct_identity(
        &pokemon.id,
        None,
        Some(Identity {
            species: "Charmander".into(),
            form: Some("Normal".into()),
        }),
    )
    .unwrap()
    .0
}

fn evolve(
    repo: &InventoryRepository,
    pokemon: &Pokemon,
    request_id: &str,
    target: &str,
) -> Result<(Pokemon, Vec<Tag>)> {
    repo.confirm_evolution(
        &pokemon.id,
        EvolutionConfirmation {
            request_id,
            expected_revision: pokemon.strategic_observation_revision,
            dataset_id: &pokemon.reference.dataset.dataset_id,
            dataset_version: &pokemon.reference.dataset.version,
            target_form_id: target,
        },
    )
}

fn observation<'a>(pokemon: &Pokemon, request_id: &'a str) -> ReobservationConfirmation<'a> {
    ReobservationConfirmation {
        request_id,
        expected_revision: pokemon.strategic_observation_revision,
        cp: Some(ObservationPatch {
            value: 2000,
            source: ObservationSource::Screenshot,
        }),
        ivs: None,
        observed_identity: pokemon.reference.identity.clone(),
        same_individual_confirmed: true,
    }
}

#[test]
fn charmander_charmeleon_charizard_keep_uuid_ivs_tags_and_appraisal() {
    let repo = repository();
    let initial = saved(&repo);
    let (initial, tags) = repo.evaluate(&initial.id).unwrap();
    let (middle, middle_tags) = evolve(&repo, &initial, FIRST, "charmeleon-normal").unwrap();
    assert_eq!(middle.id, initial.id);
    assert_eq!(middle.ivs, initial.ivs);
    assert_eq!(middle.cp, None);
    assert_eq!(middle_tags, tags);
    assert_eq!(middle.display_name, initial.display_name);
    assert_eq!(middle.created_at, initial.created_at);
    assert_eq!(middle.evaluation, initial.evaluation);
    assert_eq!(
        middle.reference.identity.as_ref().unwrap().species,
        "Charmeleon"
    );
    let (final_pokemon, _) = evolve(&repo, &middle, SECOND, "charizard-normal").unwrap();
    assert_eq!(final_pokemon.id, initial.id);
    assert_eq!(final_pokemon.ivs, initial.ivs);
    assert_eq!(final_pokemon.cp, None);
    assert_eq!(
        final_pokemon.reference.identity.as_ref().unwrap().species,
        "Charizard"
    );
    assert_eq!(repo.list().unwrap().len(), 1);
    let history = repo.history(&initial.id).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[1].before.cp.as_deref(), Some("9007199254740993"));
    assert_eq!(history[1].after.cp, None);
    assert!(!history[0].confirmed_at.is_empty());
    assert_eq!(history[0].pokemon_id, initial.id.as_str());
}

#[test]
fn options_and_cancellation_are_read_only_and_branch_needs_explicit_target() {
    let mut repo = repository();
    // A fresh repository avoids reusing a reference version with altered content.
    repo.catalog = Catalog::load(&fixture(true)).unwrap();
    let initial = saved(&repo);
    let options = repo.evolution_options(&initial.id).unwrap();
    assert_eq!(options.targets.len(), 2);
    assert!(options.unavailable_reason.is_none());
    assert_eq!(repo.get(&initial.id).unwrap(), initial);
    assert!(repo.history(&initial.id).unwrap().is_empty());
    assert!(matches!(
        evolve(&repo, &initial, FIRST, ""),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        evolve(&repo, &initial, FIRST, "charizard-normal"),
        Err(Error::InvalidInput(_))
    ));
    let (branch, _) = evolve(&repo, &initial, FIRST, "testbranch-normal").unwrap();
    assert_eq!(branch.reference.identity.unwrap().species, "Testbranch");
}

#[test]
fn empty_unknown_and_same_species_relationships_never_guess_targets() {
    let mut repo = repository();
    let initial = saved(&repo);
    repo.catalog = Catalog::bundled().unwrap();
    assert!(repo
        .evolution_options(&initial.id)
        .unwrap()
        .targets
        .is_empty());
    assert!(evolve(&repo, &initial, FIRST, "charmeleon-normal").is_err());
    let mut document: serde_json::Value = serde_json::from_slice(&fixture(false)).unwrap();
    document["forms"][1]["speciesId"] = json!("charmander");
    document["forms"][1]["name"] = json!("Other form");
    repo.catalog = Catalog::load(&serde_json::to_vec(&document).unwrap()).unwrap();
    assert!(repo
        .evolution_options(&initial.id)
        .unwrap()
        .targets
        .is_empty());
}

#[test]
fn targeted_observation_updates_only_selected_uuid_and_missing_fields_keep_values() {
    let repo = repository();
    let initial = saved(&repo);
    let another = saved(&repo);
    let (changed, tags) = repo
        .reobserve(&initial.id, observation(&initial, FIRST))
        .unwrap();
    assert_eq!(changed.id, initial.id);
    assert_eq!(changed.cp, Some(2000));
    assert_eq!(changed.ivs, initial.ivs);
    assert_eq!(changed.reference, initial.reference);
    assert_eq!(changed.display_name, initial.display_name);
    assert_eq!(tags, repo.tags_for(&initial.id).unwrap());
    assert_eq!(repo.get(&another.id).unwrap(), another);
    assert_eq!(repo.list().unwrap().len(), 2);
    let mut iv_only = observation(&changed, SECOND);
    iv_only.cp = None;
    iv_only.ivs = Some(ObservationPatch {
        value: AppraisalIvs {
            attack: 1,
            defense: 2,
            hp: 3,
        },
        source: ObservationSource::Manual,
    });
    let (changed, _) = repo.reobserve(&initial.id, iv_only).unwrap();
    assert_eq!(changed.cp, Some(2000));
    assert_eq!(changed.ivs.unwrap().attack, 1);
    let history = repo.history(&initial.id).unwrap();
    assert!(matches!(
        history[0].ivs_source,
        Some(ObservationSource::Manual)
    ));
    assert!(matches!(
        history[1].cp_source,
        Some(ObservationSource::Screenshot)
    ));
}

#[test]
fn screenshot_mismatch_and_unconfirmed_individual_reject_every_change() {
    let repo = repository();
    let initial = saved(&repo);
    let mut input = observation(&initial, FIRST);
    input.observed_identity = Some(Identity {
        species: "Charmeleon".into(),
        form: Some("Normal".into()),
    });
    assert!(matches!(
        repo.reobserve(&initial.id, input),
        Err(Error::InvalidInput(_))
    ));
    let mut input = observation(&initial, FIRST);
    input.same_individual_confirmed = false;
    assert!(matches!(
        repo.reobserve(&initial.id, input),
        Err(Error::InvalidInput(_))
    ));
    assert_eq!(repo.get(&initial.id).unwrap(), initial);
    assert!(repo.history(&initial.id).unwrap().is_empty());
}

#[test]
fn stale_evolution_and_screenshot_drafts_and_repeated_noops_are_rejected() {
    let repo = repository();
    let initial = saved(&repo);
    let (evolved, _) = evolve(&repo, &initial, FIRST, "charmeleon-normal").unwrap();
    assert!(matches!(
        evolve(&repo, &initial, FIRST, "charmeleon-normal"),
        Err(Error::AlreadyConfirmed)
    ));
    assert!(matches!(
        evolve(&repo, &initial, SECOND, "charmeleon-normal"),
        Err(Error::ObservationConflict)
    ));
    assert!(matches!(
        repo.reobserve(&initial.id, observation(&initial, SECOND)),
        Err(Error::ObservationConflict)
    ));
    let (observed, _) = repo
        .reobserve(&evolved.id, observation(&evolved, SECOND))
        .unwrap();
    let no_op = observation(&observed, THIRD);
    let (same, _) = repo.reobserve(&observed.id, no_op).unwrap();
    assert_eq!(
        same.strategic_observation_revision,
        observed.strategic_observation_revision
    );
    assert!(matches!(
        repo.reobserve(&observed.id, observation(&observed, THIRD)),
        Err(Error::AlreadyConfirmed)
    ));
    assert_eq!(repo.history(&initial.id).unwrap().len(), 3);
}

#[test]
fn evolution_and_observation_rollback_on_history_failure_and_revision_overflow() {
    let repo = repository();
    let initial = saved(&repo);
    repo.connection.execute_batch("CREATE TRIGGER fail_event BEFORE INSERT ON pokemon_lifecycle_events BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();
    assert!(evolve(&repo, &initial, FIRST, "charmeleon-normal").is_err());
    assert_eq!(repo.get(&initial.id).unwrap(), initial);
    assert!(repo
        .reobserve(&initial.id, observation(&initial, SECOND))
        .is_err());
    assert_eq!(repo.get(&initial.id).unwrap(), initial);
    assert!(repo.history(&initial.id).unwrap().is_empty());
    repo.connection
        .execute_batch("DROP TRIGGER fail_event;")
        .unwrap();
    repo.connection
        .execute(
            "UPDATE pokemon SET strategic_observation_revision = ?2 WHERE id = ?1",
            params![initial.id.as_str(), i64::MAX - 1],
        )
        .unwrap();
    let almost = repo.get(&initial.id).unwrap();
    // CP invalidation consumes one revision; identity update would overflow.
    assert!(evolve(&repo, &almost, FIRST, "charmeleon-normal").is_err());
    assert_eq!(repo.get(&initial.id).unwrap(), almost);
    assert!(repo.history(&initial.id).unwrap().is_empty());
}

#[test]
fn post_evolution_observation_never_restores_invalidated_cp_without_selected_patch() {
    let repo = repository();
    let initial = saved(&repo);
    let (evolved, _) = evolve(&repo, &initial, FIRST, "charmeleon-normal").unwrap();
    let mut input = observation(&evolved, SECOND);
    input.cp = None;
    input.ivs = Some(ObservationPatch {
        value: evolved.ivs.unwrap(),
        source: ObservationSource::Screenshot,
    });
    let (observed, _) = repo.reobserve(&evolved.id, input).unwrap();
    assert_eq!(observed.cp, None);
}

#[test]
fn reference_change_after_preview_rejects_confirmation() {
    let mut repo = repository();
    let initial = saved(&repo);
    let mut document: serde_json::Value = serde_json::from_slice(&fixture(false)).unwrap();
    document["dataset"]["version"] = json!("lifecycle-test-2");
    repo.activate_reference(&serde_json::to_vec(&document).unwrap())
        .unwrap();
    assert!(matches!(
        evolve(&repo, &initial, FIRST, "charmeleon-normal"),
        Err(Error::ObservationConflict)
    ));
    assert!(repo.history(&initial.id).unwrap().is_empty());
}

#[test]
fn equal_species_labels_still_invalidate_drafts_and_audit_actual_species_ids() {
    let mut repo = repository();
    let mut document: serde_json::Value = serde_json::from_slice(&fixture(false)).unwrap();
    document["species"][0]["name"] = json!("Shared");
    repo.catalog = Catalog::load(&serde_json::to_vec(&document).unwrap()).unwrap();
    let pokemon = repo
        .create(&PokemonInput {
            display_name: "Nickname".into(),
            cp: None,
            ivs: None,
        })
        .unwrap();
    let (before, _) = repo
        .correct_identity(
            &pokemon.id,
            None,
            Some(Identity {
                species: "Shared".into(),
                form: Some("Normal".into()),
            }),
        )
        .unwrap();
    document["dataset"]["version"] = json!("renamed-species");
    document["species"][0]["name"] = json!("Source");
    document["species"][0]["aliases"] = json!(["Shared"]);
    document["species"][1]["name"] = json!("Shared");
    repo.activate_reference(&serde_json::to_vec(&document).unwrap())
        .unwrap();
    let preview = repo.get(&pokemon.id).unwrap();
    assert_eq!(before.reference.identity, preview.reference.identity);
    let (after, _) = evolve(&repo, &preview, FIRST, "charmeleon-normal").unwrap();
    assert_eq!(after.reference.identity, preview.reference.identity);
    assert!(after.strategic_observation_revision > preview.strategic_observation_revision);
    assert!(matches!(
        repo.reobserve(&pokemon.id, observation(&preview, SECOND)),
        Err(Error::ObservationConflict)
    ));
    let event = repo.history(&pokemon.id).unwrap().remove(0);
    assert_eq!(event.before.species_id.as_deref(), Some("charmander"));
    assert_eq!(event.after.species_id.as_deref(), Some("charmeleon"));
}
