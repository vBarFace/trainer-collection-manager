use super::*;

fn repository() -> InventoryRepository {
    let mut connection = Connection::open_in_memory().unwrap();
    database::configure_connection(&connection).unwrap();
    database::apply_migrations(&mut connection).unwrap();
    InventoryRepository {
        connection,
        catalog: Catalog::bundled().unwrap(),
    }
}

fn input() -> PokemonInput {
    PokemonInput {
        display_name: "Saved individual".into(),
        cp: Some(9007199254740993),
        ivs: Some(AppraisalIvs {
            attack: 15,
            defense: 14,
            hp: 13,
        }),
    }
}

fn identity(species: &str, form: Option<&str>) -> Identity {
    Identity {
        species: species.into(),
        form: form.map(Into::into),
    }
}

fn temporary_path(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "tcm-strategic-{label}-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn appraisal_revision(repo: &InventoryRepository, id: &PokemonId) -> i64 {
    repo.connection
        .query_row(
            "SELECT appraisal_revision FROM pokemon WHERE id = ?1",
            [id.as_str()],
            |row| row.get(0),
        )
        .unwrap()
}

// Reconstruct the previous schema without rewriting any immutable migration.
fn downgrade_to_v5(connection: &Connection) {
    connection
        .execute_batch(
            "DROP TABLE pokemon_lifecycle_events;
             DROP TRIGGER pokemon_strategic_observation_changed;
             DROP TRIGGER pokemon_identity_observation_added;
             DROP TRIGGER pokemon_identity_observation_changed;
             DROP TRIGGER pokemon_identity_observation_removed;
             ALTER TABLE pokemon DROP COLUMN strategic_observation_revision;
             DELETE FROM schema_migrations WHERE version >= 6;",
        )
        .unwrap();
}

#[test]
fn strategic_cp_correction_preserves_uuid_tags_identity_and_appraisal() {
    let mut repo = repository();
    repo.activate_reference(crate::reference::tests::FIXTURE)
        .unwrap();
    let original = repo.create(&input()).unwrap();
    assert_eq!(original.strategic_observation_revision, 0);
    let tag = repo.create_custom_tag("Personal favorite").unwrap();
    repo.assign_tag(&original.id, tag.id).unwrap();
    repo.assign_tag(&original.id, repo.tags().unwrap()[0].id)
        .unwrap();
    repo.correct_identity(
        &original.id,
        None,
        Some(identity("Testling", Some("Normal"))),
    )
    .unwrap();
    let (before, tags) = repo.evaluate(&original.id).unwrap();
    let appraisal_before = appraisal_revision(&repo, &original.id);
    let (changed, assigned) = repo
        .correct_cp(
            &original.id,
            before.cp,
            before.strategic_observation_revision,
            Some(i64::MAX),
        )
        .unwrap();
    assert_eq!(changed.cp, Some(i64::MAX));
    assert_eq!(changed.strategic_observation_revision, 2);
    assert_eq!(assigned, tags);
    assert_eq!(appraisal_revision(&repo, &original.id), appraisal_before);
    let mut expected = before.clone();
    expected.cp = changed.cp;
    expected.strategic_observation_revision = changed.strategic_observation_revision;
    expected.updated_at = changed.updated_at.clone();
    assert_eq!(changed, expected);

    // Explicit unknown, restoring known CP, and confirming a no-op all preserve appraisal.
    let cleared = repo
        .correct_cp(&original.id, Some(i64::MAX), 2, None)
        .unwrap()
        .0;
    assert_eq!(cleared.cp, None);
    assert_eq!(cleared.strategic_observation_revision, 3);
    let restored = repo.correct_cp(&original.id, None, 3, before.cp).unwrap().0;
    assert_eq!(restored.strategic_observation_revision, 4);
    assert_eq!(restored.evaluation, before.evaluation);
    repo.connection
        .execute(
            "UPDATE pokemon SET updated_at = '2000-01-01 00:00:00' WHERE id = ?1",
            [original.id.as_str()],
        )
        .unwrap();
    let unchanged = repo.get(&original.id).unwrap();
    assert_eq!(
        repo.correct_cp(&original.id, unchanged.cp, 4, unchanged.cp)
            .unwrap()
            .0,
        unchanged
    );
    assert_eq!(repo.tags_for(&original.id).unwrap(), tags);
}

#[test]
fn strategic_revision_tracks_iv_and_identity_changes_but_not_refreshes_or_annotations() {
    let mut repo = repository();
    let original = repo.create(&input()).unwrap();
    let id = &original.id;
    repo.evaluate(id).unwrap();
    repo.correct_ivs(id, original.ivs, original.ivs).unwrap();
    assert_eq!(repo.get(id).unwrap().strategic_observation_revision, 0);
    let iv_changed = repo.correct_ivs(id, original.ivs, None).unwrap().0;
    assert_eq!(iv_changed.strategic_observation_revision, 1);
    assert_eq!(iv_changed.evaluation.state, EvaluationState::Stale);
    assert_eq!(appraisal_revision(&repo, id), 1);
    repo.correct_ivs(id, None, original.ivs).unwrap();
    assert_eq!(repo.get(id).unwrap().strategic_observation_revision, 2);

    let confirmed = identity("Testling", Some("Normal"));
    let added = repo
        .correct_identity(id, None, Some(confirmed.clone()))
        .unwrap()
        .0;
    assert_eq!(added.strategic_observation_revision, 3);
    assert_eq!(added.reference.state, ResolutionState::Unavailable);
    repo.correct_identity(id, Some(confirmed.clone()), Some(confirmed.clone()))
        .unwrap();
    assert_eq!(repo.get(id).unwrap().strategic_observation_revision, 3);
    repo.activate_reference(crate::reference::tests::FIXTURE)
        .unwrap();
    // Confirming the same queries can now establish pins; this is only a reference refresh.
    let resolved = repo
        .correct_identity(id, Some(confirmed.clone()), Some(confirmed.clone()))
        .unwrap()
        .0;
    assert_eq!(resolved.reference.state, ResolutionState::Resolved);
    assert_eq!(resolved.strategic_observation_revision, 3);
    let mut dataset = crate::reference::tests::document();
    dataset["dataset"]["version"] = "strategic-refresh".into();
    repo.activate_reference(&serde_json::to_vec(&dataset).unwrap())
        .unwrap();
    assert_eq!(repo.get(id).unwrap().strategic_observation_revision, 3);
    repo.connection.execute(
        "UPDATE pokemon_reference_identity SET confirmed_at = '2001-01-01 00:00:00' WHERE pokemon_id = ?1",
        [id.as_str()],
    ).unwrap();
    assert_eq!(repo.get(id).unwrap().strategic_observation_revision, 3);

    let unknown_form = identity("Testling", None);
    repo.correct_identity(id, Some(confirmed.clone()), Some(unknown_form.clone()))
        .unwrap();
    assert_eq!(repo.get(id).unwrap().strategic_observation_revision, 4);
    let other = identity("Another confirmed species", None);
    repo.correct_identity(id, Some(unknown_form), Some(other.clone()))
        .unwrap();
    assert_eq!(repo.get(id).unwrap().strategic_observation_revision, 5);
    repo.correct_identity(id, Some(other), None).unwrap();
    assert_eq!(repo.get(id).unwrap().strategic_observation_revision, 6);
    repo.correct_identity(id, None, None).unwrap();
    assert_eq!(repo.get(id).unwrap().strategic_observation_revision, 6);

    let mut renamed = input();
    renamed.display_name = "Only a display label".into();
    repo.update(id, &renamed).unwrap();
    let tag = repo.create_custom_tag("Display annotation").unwrap();
    repo.assign_tag(id, tag.id).unwrap();
    repo.rename_custom_tag(tag.id, "Renamed annotation")
        .unwrap();
    repo.remove_tag(id, tag.id).unwrap();
    repo.delete_custom_tag(tag.id).unwrap();
    repo.evaluate(id).unwrap();
    repo.snapshot().unwrap();
    repo.list().unwrap();
    assert_eq!(repo.get(id).unwrap().strategic_observation_revision, 6);
    assert_eq!(appraisal_revision(&repo, id), 2);
    // The older complete-update path must not bypass tracking. Several fields in one
    // confirmed update consume one observation revision, and one appraisal revision.
    renamed.cp = Some(100);
    renamed.ivs = None;
    let changed = repo.update(id, &renamed).unwrap();
    assert_eq!(changed.strategic_observation_revision, 7);
    assert_eq!(appraisal_revision(&repo, id), 3);
}

#[test]
fn strategic_cp_conflicts_reject_changed_reverted_and_unrelated_observation_drafts() {
    let path = temporary_path("conflicts");
    let first = InventoryRepository::open(&path).unwrap();
    let original = first.create(&input()).unwrap();
    let second = InventoryRepository::open(&path).unwrap();
    second
        .correct_cp(&original.id, original.cp, 0, Some(1))
        .unwrap();
    let changed = first.get(&original.id).unwrap();
    assert!(matches!(
        first.correct_cp(&original.id, original.cp, 0, Some(2)),
        Err(Error::ObservationConflict)
    ));
    assert_eq!(first.get(&original.id).unwrap(), changed);
    second
        .correct_cp(&original.id, Some(1), 1, original.cp)
        .unwrap();
    let reverted = first.get(&original.id).unwrap();
    assert_eq!(reverted.cp, original.cp);
    assert_eq!(reverted.strategic_observation_revision, 2);
    assert!(matches!(
        first.correct_cp(&original.id, original.cp, 0, original.cp),
        Err(Error::ObservationConflict)
    ));
    assert_eq!(first.get(&original.id).unwrap(), reverted);
    assert!(matches!(
        first.correct_cp(&original.id, Some(999), 2, Some(2)),
        Err(Error::ObservationConflict)
    ));
    second
        .correct_ivs(&original.id, original.ivs, None)
        .unwrap();
    assert!(matches!(
        first.correct_cp(&original.id, original.cp, 2, Some(2)),
        Err(Error::ObservationConflict)
    ));
    let after_ivs = first.get(&original.id).unwrap();
    second
        .correct_identity(&original.id, None, Some(identity("Explicit", None)))
        .unwrap();
    assert!(matches!(
        first.correct_cp(
            &original.id,
            original.cp,
            after_ivs.strategic_observation_revision,
            Some(2)
        ),
        Err(Error::ObservationConflict)
    ));
    drop(first);
    drop(second);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn strategic_cp_validation_and_missing_records_are_atomic() {
    let repo = repository();
    let original = repo.create(&input()).unwrap();
    for (expected, revision, corrected) in [
        (Some(0), 0, Some(1)),
        (Some(-1), 0, Some(1)),
        (original.cp, -1, Some(1)),
        (original.cp, 0, Some(0)),
        (original.cp, 0, Some(-1)),
    ] {
        assert!(matches!(
            repo.correct_cp(&original.id, expected, revision, corrected),
            Err(Error::InvalidInput(_))
        ));
        assert_eq!(repo.get(&original.id).unwrap(), original);
    }
    let missing = PokemonId::parse("00000000-0000-4000-8000-000000000000").unwrap();
    assert!(matches!(
        repo.correct_cp(&missing, None, 0, Some(1)),
        Err(Error::NotFound(_))
    ));
    repo.delete(&original.id).unwrap();
    assert!(matches!(
        repo.correct_cp(&original.id, original.cp, 0, Some(1)),
        Err(Error::NotFound(_))
    ));
    assert!(repo.list().unwrap().is_empty());
}

#[test]
fn strategic_overflow_rolls_back_cp_ivs_identity_and_appraisal_revision() {
    let repo = repository();
    let original = repo.create(&input()).unwrap();
    let confirmed = identity("Confirmed", Some("Form"));
    repo.correct_identity(&original.id, None, Some(confirmed.clone()))
        .unwrap();
    repo.evaluate(&original.id).unwrap();
    repo.connection.execute(
        "UPDATE pokemon SET strategic_observation_revision = ?2, updated_at = '2000-01-01 00:00:00' WHERE id = ?1",
        params![original.id.as_str(), i64::MAX],
    ).unwrap();
    let before = repo.get(&original.id).unwrap();
    let appraisal_before = appraisal_revision(&repo, &original.id);
    assert!(repo
        .correct_cp(&original.id, before.cp, i64::MAX, Some(1))
        .is_err());
    assert_eq!(repo.get(&original.id).unwrap(), before);
    assert!(repo.correct_ivs(&original.id, before.ivs, None).is_err());
    assert_eq!(repo.get(&original.id).unwrap(), before);
    assert_eq!(appraisal_revision(&repo, &original.id), appraisal_before);
    assert!(repo
        .correct_identity(
            &original.id,
            Some(confirmed.clone()),
            Some(identity("Changed", None))
        )
        .is_err());
    assert_eq!(repo.get(&original.id).unwrap(), before);
    assert!(repo
        .correct_identity(&original.id, Some(confirmed.clone()), None)
        .is_err());
    assert_eq!(repo.get(&original.id).unwrap(), before);
    assert_eq!(
        repo.correct_cp(&original.id, before.cp, i64::MAX, before.cp)
            .unwrap()
            .0,
        before
    );
    assert_eq!(
        repo.correct_ivs(&original.id, before.ivs, before.ivs)
            .unwrap()
            .0,
        before
    );
    assert_eq!(
        repo.correct_identity(&original.id, Some(confirmed.clone()), Some(confirmed))
            .unwrap()
            .0,
        before
    );

    let without_identity = repo.create(&input()).unwrap();
    repo.connection
        .execute(
            "UPDATE pokemon SET strategic_observation_revision = ?2 WHERE id = ?1",
            params![without_identity.id.as_str(), i64::MAX],
        )
        .unwrap();
    assert!(repo
        .correct_identity(&without_identity.id, None, Some(identity("New", None)))
        .is_err());
    let loaded = repo.get(&without_identity.id).unwrap();
    assert!(loaded.reference.identity.is_none());
    assert_eq!(loaded.strategic_observation_revision, i64::MAX);
}

#[test]
fn strategic_revision_detects_changes_with_identical_second_precision_timestamps() {
    let repo = repository();
    let mut saved = repo.create(&input()).unwrap();
    let mut observed_same_second = false;
    for cp in 1..=10 {
        let changed = repo
            .correct_cp(
                &saved.id,
                saved.cp,
                saved.strategic_observation_revision,
                Some(cp),
            )
            .unwrap()
            .0;
        assert_eq!(
            changed.strategic_observation_revision,
            saved.strategic_observation_revision + 1
        );
        if changed.updated_at == saved.updated_at {
            observed_same_second = true;
            assert!(matches!(
                repo.correct_cp(
                    &saved.id,
                    saved.cp,
                    saved.strategic_observation_revision,
                    saved.cp
                ),
                Err(Error::ObservationConflict)
            ));
            break;
        }
        saved = changed;
    }
    assert!(
        observed_same_second,
        "consecutive corrections should exercise SQLite's second-precision timestamp collision"
    );
}

#[test]
fn strategic_v5_upgrade_preserves_records_and_initializes_revision_without_reset_on_reopen() {
    let path = temporary_path("migration");
    let repo = InventoryRepository::open(&path).unwrap();
    let original = repo.create(&input()).unwrap();
    let unknown = repo
        .create(&PokemonInput {
            cp: None,
            ivs: None,
            ..input()
        })
        .unwrap();
    repo.assign_tag(&original.id, repo.tags().unwrap()[0].id)
        .unwrap();
    repo.correct_identity(&original.id, None, Some(identity("Legacy", None)))
        .unwrap();
    let (mut before, tags) = repo.evaluate(&original.id).unwrap();
    before.strategic_observation_revision = 0;
    downgrade_to_v5(&repo.connection);
    drop(repo);
    let repo = InventoryRepository::open(&path).unwrap();
    assert_eq!(repo.get(&original.id).unwrap(), before);
    assert_eq!(repo.tags_for(&original.id).unwrap(), tags);
    assert_eq!(repo.get(&unknown.id).unwrap(), unknown);
    assert_eq!(
        repo.create(&input())
            .unwrap()
            .strategic_observation_revision,
        0
    );
    let corrected = repo.correct_cp(&original.id, before.cp, 0, None).unwrap().0;
    assert_eq!(corrected.strategic_observation_revision, 1);
    let restored = repo
        .correct_cp(&original.id, None, 1, Some(i64::MAX))
        .unwrap()
        .0;
    drop(repo);
    let repo = InventoryRepository::open(&path).unwrap();
    assert_eq!(repo.get(&original.id).unwrap(), restored);
    repo.snapshot().unwrap();
    assert_eq!(
        repo.get(&original.id)
            .unwrap()
            .strategic_observation_revision,
        2
    );
    assert_eq!(repo.tags_for(&original.id).unwrap(), tags);
    drop(repo);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn strategic_failed_migration_rolls_back_column_and_version() {
    let mut repo = repository();
    let original = repo.create(&input()).unwrap();
    downgrade_to_v5(&repo.connection);
    repo.connection.execute_batch(
        "CREATE TRIGGER pokemon_strategic_observation_changed AFTER UPDATE ON pokemon BEGIN SELECT 1; END;",
    ).unwrap();
    assert!(database::apply_migrations(&mut repo.connection).is_err());
    assert!(repo
        .connection
        .prepare("SELECT strategic_observation_revision FROM pokemon")
        .is_err());
    assert_eq!(
        repo.connection
            .query_row("SELECT max(version) FROM schema_migrations", [], |row| row
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        5
    );
    assert_eq!(
        repo.connection
            .query_row(
                "SELECT cp FROM pokemon WHERE id = ?1",
                [original.id.as_str()],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        original.cp.unwrap()
    );
}
