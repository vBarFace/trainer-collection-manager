use rusqlite::{Connection, Transaction};
use std::fs;
use std::path::Path;

pub const DATABASE_FILE_NAME: &str = "trainer-collection-manager.sqlite3";

pub fn initialize(database_path: &Path) -> rusqlite::Result<()> {
    if let Some(parent) = database_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
    }

    let mut connection = Connection::open(database_path)?;
    configure_connection(&connection)?;
    apply_migrations(&mut connection)
}

pub(crate) fn configure_connection(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
}

pub(crate) fn apply_migrations(connection: &mut Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        ) STRICT;",
    )?;

    let transaction = connection.transaction()?;
    apply_migration(&transaction, 1, |transaction| {
        transaction.execute_batch(
            "CREATE TABLE application_settings (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            ) STRICT;",
        )?;
        Ok(())
    })?;
    apply_migration(&transaction, 2, |transaction| {
        transaction.execute_batch(
            "CREATE TABLE pokemon (
                id TEXT PRIMARY KEY NOT NULL CHECK (
                    length(id) = 36 AND id = lower(id) AND
                    substr(id, 9, 1) = '-' AND substr(id, 14, 1) = '-' AND
                    substr(id, 19, 1) = '-' AND substr(id, 24, 1) = '-' AND
                    length(replace(id, '-', '')) = 32 AND
                    replace(id, '-', '') NOT GLOB '*[^0-9a-f]*' AND
                    substr(id, 15, 1) = '4' AND substr(id, 20, 1) IN ('8','9','a','b')
                ),
                display_name TEXT NOT NULL CHECK (length(trim(display_name)) BETWEEN 1 AND 100),
                cp INTEGER CHECK (cp > 0),
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            ) STRICT;
            CREATE TRIGGER pokemon_id_immutable BEFORE UPDATE OF id ON pokemon
                WHEN NEW.id != OLD.id BEGIN SELECT RAISE(ABORT, 'Pokemon UUID is immutable'); END;
            CREATE TABLE tags (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL COLLATE NOCASE UNIQUE
                    CHECK (name = trim(name) AND length(name) BETWEEN 1 AND 50),
                scope TEXT NOT NULL CHECK (scope IN ('standard', 'local')),
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            ) STRICT;
            INSERT INTO tags (name, scope) VALUES
                ('EVOLVE', 'standard'), ('POWER UP', 'standard'),
                ('ATTACKER', 'standard'), ('GYM DEF', 'standard'),
                ('ROCKET', 'standard'), ('PVP 1500', 'standard'),
                ('PVP 2500', 'standard'), ('TRANSFER', 'standard');
            CREATE TRIGGER standard_tag_update BEFORE UPDATE ON tags
                WHEN OLD.scope = 'standard' OR NEW.scope = 'standard'
                BEGIN SELECT RAISE(ABORT, 'Standard tags are immutable'); END;
            CREATE TRIGGER standard_tag_delete BEFORE DELETE ON tags
                WHEN OLD.scope = 'standard'
                BEGIN SELECT RAISE(ABORT, 'Standard tags are immutable'); END;
            CREATE TRIGGER standard_tag_insert BEFORE INSERT ON tags
                WHEN NEW.scope = 'standard'
                BEGIN SELECT RAISE(ABORT, 'Standard tags are migration-owned'); END;
            CREATE TABLE pokemon_tags (
                pokemon_id TEXT NOT NULL REFERENCES pokemon(id) ON DELETE CASCADE,
                tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (pokemon_id, tag_id)
            ) STRICT;
            CREATE INDEX pokemon_tags_by_tag ON pokemon_tags(tag_id, pokemon_id);",
        )
    })?;
    apply_migration(&transaction, 3, |transaction| {
        transaction.execute_batch(
            "ALTER TABLE pokemon ADD COLUMN iv_attack INTEGER CHECK (iv_attack BETWEEN 0 AND 15);
            ALTER TABLE pokemon ADD COLUMN iv_defense INTEGER CHECK (iv_defense BETWEEN 0 AND 15);
            ALTER TABLE pokemon ADD COLUMN iv_hp INTEGER CHECK (iv_hp BETWEEN 0 AND 15)
                CHECK (
                    (iv_attack IS NULL AND iv_defense IS NULL AND iv_hp IS NULL) OR
                    (iv_attack IS NOT NULL AND iv_defense IS NOT NULL AND iv_hp IS NOT NULL)
                );
            DROP TRIGGER standard_tag_insert;
            DROP TRIGGER standard_tag_update;
            INSERT INTO tags (name, scope) VALUES ('SAVE/SPECIAL', 'standard')
                ON CONFLICT(name) DO UPDATE SET name = 'SAVE/SPECIAL', scope = 'standard';
            CREATE TRIGGER standard_tag_update BEFORE UPDATE ON tags
                WHEN OLD.scope = 'standard' OR NEW.scope = 'standard'
                BEGIN SELECT RAISE(ABORT, 'Standard tags are immutable'); END;
            CREATE TRIGGER standard_tag_insert BEFORE INSERT ON tags
                WHEN NEW.scope = 'standard'
                BEGIN SELECT RAISE(ABORT, 'Standard tags are migration-owned'); END;",
        )
    })?;
    apply_migration(&transaction, 4, |transaction| {
        transaction.execute_batch(
            "ALTER TABLE pokemon ADD COLUMN appraisal_revision INTEGER NOT NULL DEFAULT 0
                CHECK (appraisal_revision >= 0);
            CREATE TABLE pokemon_appraisals (
                pokemon_id TEXT PRIMARY KEY NOT NULL REFERENCES pokemon(id) ON DELETE CASCADE,
                input_revision INTEGER NOT NULL CHECK (input_revision >= 0),
                ruleset_version TEXT NOT NULL CHECK (length(trim(ruleset_version)) > 0),
                evaluated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            ) STRICT;
            CREATE TRIGGER pokemon_appraisal_changed AFTER UPDATE OF iv_attack, iv_defense, iv_hp ON pokemon
                WHEN NEW.iv_attack IS NOT OLD.iv_attack OR NEW.iv_defense IS NOT OLD.iv_defense
                    OR NEW.iv_hp IS NOT OLD.iv_hp
                BEGIN
                    UPDATE pokemon SET appraisal_revision = OLD.appraisal_revision + 1 WHERE id = NEW.id;
                END;",
        )
    })?;
    apply_migration(&transaction, 5, |transaction| {
        transaction.execute_batch(
            "CREATE TABLE pokemon_reference_identity (
                pokemon_id TEXT PRIMARY KEY NOT NULL REFERENCES pokemon(id) ON DELETE CASCADE,
                species_query TEXT NOT NULL CHECK (length(species_query) BETWEEN 1 AND 100 AND species_query = trim(species_query)),
                form_query TEXT CHECK (length(form_query) BETWEEN 1 AND 100 AND form_query = trim(form_query)),
                dataset_id TEXT,
                species_id TEXT,
                form_id TEXT,
                confirmed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                CHECK ((dataset_id IS NULL AND species_id IS NULL AND form_id IS NULL) OR
                    (dataset_id IS NOT NULL AND species_id IS NOT NULL AND form_id IS NOT NULL AND form_query IS NOT NULL))
            ) STRICT;",
        )
    })?;
    apply_migration(&transaction, 6, |transaction| {
        transaction.execute_batch(
            "ALTER TABLE pokemon ADD COLUMN strategic_observation_revision INTEGER NOT NULL DEFAULT 0
                CHECK (strategic_observation_revision >= 0);
            CREATE TRIGGER pokemon_strategic_observation_changed
                AFTER UPDATE OF cp, iv_attack, iv_defense, iv_hp ON pokemon
                WHEN NEW.cp IS NOT OLD.cp OR NEW.iv_attack IS NOT OLD.iv_attack
                    OR NEW.iv_defense IS NOT OLD.iv_defense OR NEW.iv_hp IS NOT OLD.iv_hp
                BEGIN
                    SELECT CASE WHEN OLD.strategic_observation_revision = 9223372036854775807
                        THEN RAISE(ABORT, 'Strategic observation revision overflow') END;
                    UPDATE pokemon SET strategic_observation_revision = OLD.strategic_observation_revision + 1
                        WHERE id = NEW.id;
                END;
            CREATE TRIGGER pokemon_identity_observation_added AFTER INSERT ON pokemon_reference_identity
                BEGIN
                    SELECT CASE WHEN (SELECT strategic_observation_revision FROM pokemon WHERE id = NEW.pokemon_id) = 9223372036854775807
                        THEN RAISE(ABORT, 'Strategic observation revision overflow') END;
                    UPDATE pokemon SET strategic_observation_revision = strategic_observation_revision + 1
                        WHERE id = NEW.pokemon_id;
                END;
            CREATE TRIGGER pokemon_identity_observation_changed
                AFTER UPDATE OF species_query, form_query ON pokemon_reference_identity
                WHEN NEW.species_query IS NOT OLD.species_query OR NEW.form_query IS NOT OLD.form_query
                BEGIN
                    SELECT CASE WHEN (SELECT strategic_observation_revision FROM pokemon WHERE id = NEW.pokemon_id) = 9223372036854775807
                        THEN RAISE(ABORT, 'Strategic observation revision overflow') END;
                    UPDATE pokemon SET strategic_observation_revision = strategic_observation_revision + 1
                        WHERE id = NEW.pokemon_id;
                END;
            CREATE TRIGGER pokemon_identity_observation_removed AFTER DELETE ON pokemon_reference_identity
                BEGIN
                    SELECT CASE WHEN (SELECT strategic_observation_revision FROM pokemon WHERE id = OLD.pokemon_id) = 9223372036854775807
                        THEN RAISE(ABORT, 'Strategic observation revision overflow') END;
                    UPDATE pokemon SET strategic_observation_revision = strategic_observation_revision + 1
                        WHERE id = OLD.pokemon_id;
                END;",
        )
    })?;
    transaction.commit()
}

fn apply_migration<F>(
    transaction: &Transaction<'_>,
    version: i64,
    migration: F,
) -> rusqlite::Result<()>
where
    F: FnOnce(&Transaction<'_>) -> rusqlite::Result<()>,
{
    let already_applied = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
        [version],
        |row| row.get::<_, bool>(0),
    )?;

    if !already_applied {
        migration(transaction)?;
        transaction.execute(
            "INSERT INTO schema_migrations (version) VALUES (?1)",
            [version],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_migration_rejects_conflicts_without_recording_version() {
        let mut connection = Connection::open_in_memory().unwrap();
        configure_connection(&connection).unwrap();
        apply_migrations(&mut connection).unwrap();
        connection.execute_batch("DROP TABLE pokemon_reference_identity; DELETE FROM schema_migrations WHERE version >= 5; CREATE TABLE pokemon_reference_identity (conflict TEXT);").unwrap();
        assert!(apply_migrations(&mut connection).is_err());
        assert_eq!(
            connection
                .query_row("SELECT max(version) FROM schema_migrations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            4
        );
        assert!(connection
            .prepare("SELECT conflict FROM pokemon_reference_identity")
            .is_ok());
        assert!(connection
            .prepare("SELECT appraisal_revision FROM pokemon")
            .is_ok());
    }

    #[test]
    fn version_one_upgrade_preserves_settings_and_is_idempotent() {
        let mut connection = Connection::open_in_memory().unwrap();
        configure_connection(&connection).unwrap();
        // The exact schema produced by Task 001, before inventory existed.
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                version INTEGER PRIMARY KEY NOT NULL,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            ) STRICT;
            INSERT INTO schema_migrations (version) VALUES (1);
            CREATE TABLE application_settings (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            ) STRICT;
            INSERT INTO application_settings (key, value) VALUES ('example', 'preserved');",
            )
            .unwrap();
        apply_migrations(&mut connection).unwrap();
        apply_migrations(&mut connection).unwrap();
        assert_eq!(
            connection
                .query_row(
                    "SELECT value FROM application_settings WHERE key = 'example'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "preserved"
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM schema_migrations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            6
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM tags", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            9
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM pokemon", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn failed_inventory_migration_rolls_back_without_recording_version() {
        let mut connection = Connection::open_in_memory().unwrap();
        // Simulate a conflicting table: failure must not leave half a migration.
        connection
            .execute_batch("CREATE TABLE tags (id INTEGER);")
            .unwrap();
        assert!(apply_migrations(&mut connection).is_err());
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM schema_migrations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(connection.query_row("SELECT count(*) FROM sqlite_master WHERE name IN ('pokemon', 'application_settings')", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
    }

    #[test]
    fn migrations_create_metadata_schema_and_are_idempotent() {
        let temporary_directory = std::env::temp_dir().join(format!(
            "tcm-db-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after the Unix epoch")
                .as_nanos(),
        ));
        let database_path = temporary_directory.join(DATABASE_FILE_NAME);

        initialize(&database_path).expect("initial migration should succeed");
        initialize(&database_path).expect("reapplying migrations should succeed");

        let connection = Connection::open(database_path).expect("database should open");
        let version: i64 = connection
            .query_row("SELECT max(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("migration version should be recorded");
        assert_eq!(version, 6);
        drop(connection);

        fs::remove_dir_all(temporary_directory).expect("temporary database should be removable");
    }

    #[test]
    fn failed_appraisal_upgrade_rolls_back_revision_column_and_version() {
        let mut connection = Connection::open_in_memory().unwrap();
        configure_connection(&connection).unwrap();
        connection
            .execute_batch(include_str!("../tests/fixtures/schema-v2.sql"))
            .unwrap();
        connection
            .execute_batch(include_str!("../tests/fixtures/migration-v3.sql"))
            .unwrap();
        // Conflict after ALTER TABLE must not leave a partially applied upgrade.
        connection
            .execute_batch("CREATE TABLE pokemon_appraisals (conflict TEXT);")
            .unwrap();
        assert!(apply_migrations(&mut connection).is_err());
        assert!(connection
            .prepare("SELECT appraisal_revision FROM pokemon")
            .is_err());
        assert_eq!(
            connection
                .query_row("SELECT max(version) FROM schema_migrations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            3
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM tags", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            9
        );
    }
}
