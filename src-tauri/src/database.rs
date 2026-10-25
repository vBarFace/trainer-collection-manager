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

fn configure_connection(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
}

fn apply_migrations(connection: &mut Connection) -> rusqlite::Result<()> {
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
    transaction.commit()
}

fn apply_migration<F>(transaction: &Transaction<'_>, version: i64, migration: F) -> rusqlite::Result<()>
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
        transaction.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [version])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
            .query_row("SELECT version FROM schema_migrations", [], |row| row.get(0))
            .expect("migration version should be recorded");
        assert_eq!(version, 1);
        drop(connection);

        fs::remove_dir_all(temporary_directory).expect("temporary database should be removable");
    }
}
