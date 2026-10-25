//! Native-only inventory API. No SQL or database path is exposed to the renderer.
use crate::database;
use rusqlite::{params, Connection, OptionalExtension};
use std::{fmt, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PokemonId(String);

impl PokemonId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(value: &str) -> Result<Self> {
        let bytes = value.as_bytes();
        if bytes.len() != 36
            || !bytes.iter().enumerate().all(|(i, b)| match i {
                8 | 13 | 18 | 23 => *b == b'-',
                _ => b.is_ascii_digit() || (b'a'..=b'f').contains(b),
            })
            || bytes[14] != b'4'
            || !b"89ab".contains(&bytes[19])
        {
            return Err(Error::InvalidInput(
                "Expected a canonical lowercase UUID v4",
            ));
        }
        Ok(Self(value.into()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagId(i64);

/// Appraisal values, distinct from species base stats and observed HP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppraisalIvs {
    pub attack: u8,
    pub defense: u8,
    pub hp: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pokemon {
    pub id: PokemonId,
    pub display_name: String,
    pub cp: Option<i64>,
    pub ivs: Option<AppraisalIvs>,
    pub created_at: String,
    pub updated_at: String,
}

/// A complete replacement of editable fields; identity and timestamps are native-owned.
#[derive(Debug, Clone)]
pub struct PokemonInput {
    pub display_name: String,
    pub cp: Option<i64>,
    pub ivs: Option<AppraisalIvs>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagScope {
    Standard,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub scope: TagScope,
    pub created_at: String,
}

#[derive(Debug)]
pub enum Error {
    InvalidInput(&'static str),
    NotFound(&'static str),
    DuplicateTag,
    StandardTagImmutable,
    Storage(rusqlite::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "{message}"),
            Self::NotFound(entity) => write!(f, "{entity} not found"),
            Self::DuplicateTag => write!(f, "A tag with this name already exists"),
            Self::StandardTagImmutable => write!(f, "Standard tags cannot be changed"),
            Self::Storage(error) => write!(f, "Local inventory storage error: {error}"),
        }
    }
}

impl std::error::Error for Error {}
impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(error)
    }
}
pub type Result<T> = std::result::Result<T, Error>;

fn validate_text(value: &str, max: usize) -> Result<()> {
    if value.is_empty()
        || value.trim() != value
        || value.chars().count() > max
        || value.chars().any(char::is_control)
    {
        return Err(Error::InvalidInput(
            "Text must be nonempty, within its length limit, trimmed, and contain no control characters",
        ));
    }
    Ok(())
}

fn validate_pokemon(input: &PokemonInput) -> Result<()> {
    validate_text(&input.display_name, 100)?;
    if input.cp.is_some_and(|cp| cp <= 0) {
        return Err(Error::InvalidInput("CP must be positive or absent"));
    }
    if input
        .ivs
        .is_some_and(|ivs| ivs.attack > 15 || ivs.defense > 15 || ivs.hp > 15)
    {
        return Err(Error::InvalidInput(
            "Each appraisal IV must be between 0 and 15",
        ));
    }
    Ok(())
}

fn read_pokemon(row: &rusqlite::Row<'_>) -> rusqlite::Result<Pokemon> {
    Ok(Pokemon {
        id: PokemonId(row.get(0)?),
        display_name: row.get(1)?,
        cp: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        ivs: match (
            row.get::<_, Option<u8>>(5)?,
            row.get::<_, Option<u8>>(6)?,
            row.get::<_, Option<u8>>(7)?,
        ) {
            (None, None, None) => None,
            (Some(attack), Some(defense), Some(hp))
                if attack <= 15 && defense <= 15 && hp <= 15 =>
            {
                Some(AppraisalIvs {
                    attack,
                    defense,
                    hp,
                })
            }
            _ => return Err(rusqlite::Error::InvalidQuery),
        },
    })
}

fn read_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: TagId(row.get(0)?),
        name: row.get(1)?,
        scope: if row.get::<_, String>(2)? == "standard" {
            TagScope::Standard
        } else {
            TagScope::Local
        },
        created_at: row.get(3)?,
    })
}

/// Open only with the app-data path resolved by native startup. Each connection
/// enables foreign keys before migrations or repository operations.
pub struct InventoryRepository {
    connection: Connection,
}

impl InventoryRepository {
    pub fn open(path: &Path) -> Result<Self> {
        database::initialize(path)?;
        let connection = Connection::open(path)?;
        database::configure_connection(&connection)?;
        Ok(Self { connection })
    }

    pub fn create(&self, input: &PokemonInput) -> Result<Pokemon> {
        validate_pokemon(input)?;
        // SQLite's native PRNG supplies identity entropy; no external identifiers.
        let mut bytes: Vec<u8> = self
            .connection
            .query_row("SELECT randomblob(16)", [], |r| r.get(0))?;
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        let id = PokemonId(format!(
            "{}-{}-{}-{}-{}",
            &hex[..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..]
        ));
        self.connection.execute(
            "INSERT INTO pokemon (id, display_name, cp, iv_attack, iv_defense, iv_hp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id.0, input.display_name, input.cp, input.ivs.map(|v| v.attack), input.ivs.map(|v| v.defense), input.ivs.map(|v| v.hp)],
        )?;
        self.get(&id)
    }

    pub fn get(&self, id: &PokemonId) -> Result<Pokemon> {
        self.connection
            .query_row(
                "SELECT id, display_name, cp, created_at, updated_at, iv_attack, iv_defense, iv_hp FROM pokemon WHERE id = ?1",
                [&id.0],
                read_pokemon,
            )
            .optional()?
            .ok_or(Error::NotFound("Pokemon"))
    }

    /// Deterministic ordering; no domain ranking is implied.
    pub fn list(&self) -> Result<Vec<Pokemon>> {
        let mut statement = self.connection.prepare(
            "SELECT id, display_name, cp, created_at, updated_at, iv_attack, iv_defense, iv_hp FROM pokemon ORDER BY created_at, id",
        )?;
        let records = statement
            .query_map([], read_pokemon)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(records)
    }

    pub fn update(&self, id: &PokemonId, input: &PokemonInput) -> Result<Pokemon> {
        validate_pokemon(input)?;
        if self.connection.execute(
            "UPDATE pokemon SET display_name = ?2, cp = ?3, iv_attack = ?4, iv_defense = ?5, iv_hp = ?6, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![id.0, input.display_name, input.cp, input.ivs.map(|v| v.attack), input.ivs.map(|v| v.defense), input.ivs.map(|v| v.hp)],
        )? == 0 {
            return Err(Error::NotFound("Pokemon"));
        }
        self.get(id)
    }

    pub fn delete(&self, id: &PokemonId) -> Result<()> {
        if self
            .connection
            .execute("DELETE FROM pokemon WHERE id = ?1", [&id.0])?
            == 0
        {
            return Err(Error::NotFound("Pokemon"));
        }
        Ok(())
    }

    pub fn tags(&self) -> Result<Vec<Tag>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, name, scope, created_at FROM tags ORDER BY id")?;
        let tags = statement
            .query_map([], read_tag)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(tags)
    }

    pub fn create_custom_tag(&self, name: &str) -> Result<Tag> {
        validate_text(name, 50)?;
        let inserted = self.connection.execute(
            "INSERT INTO tags (name, scope) VALUES (?1, 'local') ON CONFLICT(name) DO NOTHING",
            [name],
        )?;
        if inserted == 0 {
            return Err(Error::DuplicateTag);
        }
        let id = TagId(self.connection.last_insert_rowid());
        self.require_tag(id)
    }

    fn require_tag(&self, id: TagId) -> Result<Tag> {
        self.connection
            .query_row(
                "SELECT id, name, scope, created_at FROM tags WHERE id = ?1",
                [id.0],
                read_tag,
            )
            .optional()?
            .ok_or(Error::NotFound("Tag"))
    }

    pub fn rename_custom_tag(&self, id: TagId, name: &str) -> Result<Tag> {
        validate_text(name, 50)?;
        if self.require_tag(id)?.scope == TagScope::Standard {
            return Err(Error::StandardTagImmutable);
        }
        if self.connection.execute(
            "UPDATE OR IGNORE tags SET name = ?2 WHERE id = ?1",
            params![id.0, name],
        )? == 0
        {
            return Err(Error::DuplicateTag);
        }
        self.require_tag(id)
    }

    pub fn delete_custom_tag(&self, id: TagId) -> Result<()> {
        if self.require_tag(id)?.scope == TagScope::Standard {
            return Err(Error::StandardTagImmutable);
        }
        self.connection
            .execute("DELETE FROM tags WHERE id = ?1", [id.0])?;
        Ok(())
    }

    /// Assignment is idempotent; duplicate requests preserve the original timestamp.
    pub fn assign_tag(&self, pokemon: &PokemonId, tag: TagId) -> Result<()> {
        self.get(pokemon)?;
        self.require_tag(tag)?;
        self.connection.execute(
            "INSERT INTO pokemon_tags (pokemon_id, tag_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
            params![pokemon.0, tag.0],
        )?;
        Ok(())
    }

    pub fn remove_tag(&self, pokemon: &PokemonId, tag: TagId) -> Result<()> {
        self.get(pokemon)?;
        self.require_tag(tag)?;
        self.connection.execute(
            "DELETE FROM pokemon_tags WHERE pokemon_id = ?1 AND tag_id = ?2",
            params![pokemon.0, tag.0],
        )?;
        Ok(())
    }

    pub fn tags_for(&self, pokemon: &PokemonId) -> Result<Vec<Tag>> {
        self.get(pokemon)?;
        let mut statement = self
            .connection
            .prepare("SELECT t.id, t.name, t.scope, t.created_at FROM tags t JOIN pokemon_tags pt ON pt.tag_id = t.id WHERE pt.pokemon_id = ?1 ORDER BY t.id")?;
        let tags = statement
            .query_map([&pokemon.0], read_tag)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> InventoryRepository {
        let mut connection = Connection::open_in_memory().unwrap();
        database::configure_connection(&connection).unwrap();
        database::apply_migrations(&mut connection).unwrap();
        InventoryRepository { connection }
    }

    fn input(name: &str) -> PokemonInput {
        PokemonInput {
            display_name: name.into(),
            cp: Some(123),
            ivs: None,
        }
    }

    #[test]
    fn ivs_round_trip_update_clear_and_reject_out_of_range_values() {
        let repo = repository();
        for ivs in [
            AppraisalIvs {
                attack: 0,
                defense: 0,
                hp: 0,
            },
            AppraisalIvs {
                attack: 15,
                defense: 15,
                hp: 15,
            },
            AppraisalIvs {
                attack: 12,
                defense: 9,
                hp: 14,
            },
        ] {
            let mut data = input("Appraised");
            data.cp = None;
            data.ivs = Some(ivs);
            let created = repo.create(&data).unwrap();
            assert_eq!(created.ivs, Some(ivs));
            assert_eq!(repo.get(&created.id).unwrap(), created);
            assert!(repo.list().unwrap().contains(&created));
            data.ivs = Some(AppraisalIvs {
                attack: 1,
                defense: 2,
                hp: 3,
            });
            let updated = repo.update(&created.id, &data).unwrap();
            assert_eq!(updated.ivs, data.ivs);
            assert_eq!(updated.id, created.id);
            assert_eq!(updated.created_at, created.created_at);
            data.ivs = None;
            assert_eq!(repo.update(&created.id, &data).unwrap().ivs, None);
        }
        let existing = repo.create(&input("Unchanged")).unwrap();
        for ivs in [
            AppraisalIvs {
                attack: 16,
                defense: 0,
                hp: 0,
            },
            AppraisalIvs {
                attack: 0,
                defense: 255,
                hp: 0,
            },
            AppraisalIvs {
                attack: 0,
                defense: 0,
                hp: 16,
            },
        ] {
            let mut data = input("Invalid");
            data.ivs = Some(ivs);
            assert!(matches!(repo.create(&data), Err(Error::InvalidInput(_))));
            assert!(matches!(
                repo.update(&existing.id, &data),
                Err(Error::InvalidInput(_))
            ));
            assert_eq!(repo.get(&existing.id).unwrap(), existing);
        }
    }

    #[test]
    fn sql_rejects_partial_and_out_of_range_ivs_on_insert_and_update() {
        let repo = repository();
        let existing = repo.create(&input("Unchanged")).unwrap();
        let mut invalid = vec![
            (Some(-1), Some(0), Some(0)),
            (Some(0), Some(-1), Some(0)),
            (Some(0), Some(0), Some(-1)),
            (Some(16), Some(0), Some(0)),
            (Some(0), Some(16), Some(0)),
            (Some(0), Some(0), Some(16)),
        ];
        for mask in 1..7 {
            invalid.push((
                if mask & 1 != 0 { Some(10) } else { None },
                if mask & 2 != 0 { Some(10) } else { None },
                if mask & 4 != 0 { Some(10) } else { None },
            ));
        }
        for (attack, defense, hp) in invalid {
            assert!(repo.connection.execute(
                "INSERT INTO pokemon (id, display_name, iv_attack, iv_defense, iv_hp) VALUES ('00000000-0000-4000-8000-000000000000', 'Invalid', ?1, ?2, ?3)",
                params![attack, defense, hp],
            ).is_err());
            assert!(repo
                .connection
                .execute(
                    "UPDATE pokemon SET iv_attack = ?2, iv_defense = ?3, iv_hp = ?4 WHERE id = ?1",
                    params![existing.id.0, attack, defense, hp],
                )
                .is_err());
            assert_eq!(repo.get(&existing.id).unwrap(), existing);
        }
    }

    #[test]
    fn version_two_upgrade_preserves_records_and_existing_special_tag_assignments() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::configure_connection(&connection).unwrap();
        connection
            .execute_batch(include_str!("../tests/fixtures/schema-v2.sql"))
            .unwrap();
        connection.execute_batch(
            "INSERT INTO pokemon (id, display_name, cp, created_at, updated_at)
                VALUES ('00000000-0000-4000-8000-000000000000', 'Legacy', 123, '2026-01-01 00:00:00', '2026-01-02 00:00:00');
            INSERT INTO tags (name, scope) VALUES ('save/special', 'local');
            INSERT INTO pokemon_tags (pokemon_id, tag_id) SELECT '00000000-0000-4000-8000-000000000000', id FROM tags WHERE name = 'save/special';
            INSERT INTO pokemon_tags (pokemon_id, tag_id) VALUES ('00000000-0000-4000-8000-000000000000', 1);",
        ).unwrap();
        let original_tag: (i64, String) = connection
            .query_row(
                "SELECT id, created_at FROM tags WHERE name = 'save/special'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        database::apply_migrations(&mut connection).unwrap();
        database::apply_migrations(&mut connection).unwrap();
        let repo = InventoryRepository { connection };
        let id = PokemonId::parse("00000000-0000-4000-8000-000000000000").unwrap();
        let pokemon = repo.get(&id).unwrap();
        assert_eq!(pokemon.display_name, "Legacy");
        assert_eq!(pokemon.cp, Some(123));
        assert_eq!(pokemon.ivs, None);
        assert_eq!(pokemon.created_at, "2026-01-01 00:00:00");
        assert_eq!(pokemon.updated_at, "2026-01-02 00:00:00");
        let tags = repo.tags_for(&id).unwrap();
        assert_eq!(tags.len(), 2);
        let special = tags.iter().find(|t| t.name == "SAVE/SPECIAL").unwrap();
        assert_eq!(special.id, TagId(original_tag.0));
        assert_eq!(special.created_at, original_tag.1);
        assert_eq!(special.scope, TagScope::Standard);
        assert!(matches!(
            repo.rename_custom_tag(special.id, "Changed"),
            Err(Error::StandardTagImmutable)
        ));
        assert!(matches!(
            repo.delete_custom_tag(special.id),
            Err(Error::StandardTagImmutable)
        ));
        assert_eq!(repo.tags().unwrap().len(), 9);
        assert_eq!(
            repo.connection
                .query_row("SELECT count(*) FROM schema_migrations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            3
        );
    }

    #[test]
    fn failed_version_three_upgrade_restores_columns_versions_and_tag_guards() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::configure_connection(&connection).unwrap();
        connection
            .execute_batch(include_str!("../tests/fixtures/schema-v2.sql"))
            .unwrap();
        // Force a late failure after the migration has replaced tag guards.
        connection
            .execute_batch("CREATE TRIGGER reject_special BEFORE INSERT ON tags WHEN NEW.name = 'SAVE/SPECIAL' BEGIN SELECT RAISE(ABORT, 'Simulated seeding failure'); END;")
            .unwrap();
        assert!(database::apply_migrations(&mut connection).is_err());
        assert_eq!(
            connection
                .query_row("SELECT max(version) FROM schema_migrations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert!(connection
            .prepare("SELECT iv_attack, iv_defense, iv_hp FROM pokemon")
            .is_err());
        assert!(connection
            .execute("UPDATE tags SET name = 'Changed' WHERE id = 1", [])
            .is_err());
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM tags", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            8
        );
        assert!(connection
            .execute(
                "INSERT INTO tags (name, scope) VALUES ('NEW', 'standard')",
                []
            )
            .is_err());
    }

    #[test]
    fn crud_preserves_uuid_and_creation_time() {
        let repo = repository();
        let first = repo.create(&input("Eevee")).unwrap();
        let second = repo.create(&input("Eevee")).unwrap();
        assert_ne!(first.id, second.id);
        assert_eq!(PokemonId::parse(first.id.as_str()).unwrap(), first.id);
        assert_eq!(repo.get(&first.id).unwrap(), first);
        assert_eq!(repo.list().unwrap().len(), 2);
        repo.connection
            .execute(
                "UPDATE pokemon SET updated_at = '2000-01-01 00:00:00' WHERE id = ?1",
                [&first.id.0],
            )
            .unwrap();
        let updated = repo
            .update(
                &first.id,
                &PokemonInput {
                    display_name: "Buddy".into(),
                    cp: None,
                    ivs: None,
                },
            )
            .unwrap();
        assert_eq!(updated.id, first.id);
        assert_eq!(updated.created_at, first.created_at);
        assert_eq!(updated.display_name, "Buddy");
        assert_eq!(updated.cp, None);
        assert!(!updated.updated_at.is_empty());
        assert_ne!(updated.updated_at, "2000-01-01 00:00:00");
        assert!(repo
            .connection
            .execute(
                "UPDATE pokemon SET id = ?1 WHERE id = ?2",
                params!["00000000-0000-4000-8000-000000000000", first.id.0]
            )
            .is_err());
        repo.delete(&first.id).unwrap();
        assert!(matches!(
            repo.get(&first.id),
            Err(Error::NotFound("Pokemon"))
        ));
        assert!(matches!(
            repo.update(&first.id, &input("Missing")),
            Err(Error::NotFound(_))
        ));
        assert!(matches!(repo.delete(&first.id), Err(Error::NotFound(_))));
        assert_eq!(repo.list().unwrap(), vec![second]);
    }

    #[test]
    fn validates_names_cp_and_uuid() {
        let repo = repository();
        for name in [
            "",
            " ",
            " Buddy",
            "Buddy ",
            "a\nb",
            "a\0b",
            &"x".repeat(101),
        ] {
            assert!(matches!(
                repo.create(&input(name)),
                Err(Error::InvalidInput(_))
            ));
        }
        for cp in [0, -1] {
            assert!(matches!(
                repo.create(&PokemonInput {
                    display_name: "Buddy".into(),
                    cp: Some(cp),
                    ivs: None,
                }),
                Err(Error::InvalidInput(_))
            ));
        }
        assert!(repo.create(&input(&"é".repeat(100))).is_ok());
        for id in [
            "",
            "not-a-uuid",
            "00000000-0000-4000-7000-000000000000",
            "00000000-0000-1000-8000-000000000000",
            "00000000-0000-4000-8000-00000000000A",
        ] {
            assert!(PokemonId::parse(id).is_err());
        }
    }

    #[test]
    fn standard_tags_and_custom_scope_are_enforced() {
        let mut repo = repository();
        database::apply_migrations(&mut repo.connection).unwrap();
        let tags = repo.tags().unwrap();
        assert_eq!(
            tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
            vec![
                "EVOLVE",
                "POWER UP",
                "ATTACKER",
                "GYM DEF",
                "ROCKET",
                "PVP 1500",
                "PVP 2500",
                "TRANSFER",
                "SAVE/SPECIAL"
            ]
        );
        assert!(tags.iter().all(|t| t.scope == TagScope::Standard));
        for tag in tags {
            assert!(matches!(
                repo.rename_custom_tag(tag.id, "New"),
                Err(Error::StandardTagImmutable)
            ));
            assert!(matches!(
                repo.delete_custom_tag(tag.id),
                Err(Error::StandardTagImmutable)
            ));
        }
        for name in ["", " ", " custom", "custom ", "a\tb", &"x".repeat(51)] {
            assert!(matches!(
                repo.create_custom_tag(name),
                Err(Error::InvalidInput(_))
            ));
        }
        assert!(matches!(
            repo.create_custom_tag("evolve"),
            Err(Error::DuplicateTag)
        ));
        assert!(matches!(
            repo.create_custom_tag("save/special"),
            Err(Error::DuplicateTag)
        ));
        let custom = repo.create_custom_tag("Favorites").unwrap();
        assert_eq!(custom.scope, TagScope::Local);
        assert!(matches!(
            repo.create_custom_tag("FAVORITES"),
            Err(Error::DuplicateTag)
        ));
        assert!(matches!(
            repo.rename_custom_tag(custom.id, "rocket"),
            Err(Error::DuplicateTag)
        ));
        assert_eq!(
            repo.rename_custom_tag(custom.id, "Keep").unwrap().name,
            "Keep"
        );
        assert!(repo
            .connection
            .execute(
                "UPDATE tags SET name = 'Changed' WHERE scope = 'standard'",
                []
            )
            .is_err());
        assert!(repo
            .connection
            .execute("DELETE FROM tags WHERE scope = 'standard'", [])
            .is_err());
        assert!(repo
            .connection
            .execute(
                "INSERT INTO tags (name, scope) VALUES ('NEW', 'standard')",
                []
            )
            .is_err());
        assert!(repo
            .connection
            .execute(
                "UPDATE tags SET scope = 'standard' WHERE id = ?1",
                [custom.id.0]
            )
            .is_err());
        repo.delete_custom_tag(custom.id).unwrap();
    }

    #[test]
    fn assignments_are_unique_and_foreign_keys_cascade() {
        let repo = repository();
        let pokemon = repo.create(&input("Buddy")).unwrap();
        let tag = repo.create_custom_tag("Keep").unwrap();
        assert!(repo.tags_for(&pokemon.id).unwrap().is_empty());
        repo.assign_tag(&pokemon.id, tag.id).unwrap();
        repo.assign_tag(&pokemon.id, tag.id).unwrap();
        assert_eq!(repo.tags_for(&pokemon.id).unwrap(), vec![tag.clone()]);
        let standard = repo.tags().unwrap()[0].id;
        repo.assign_tag(&pokemon.id, standard).unwrap();
        assert_eq!(repo.tags_for(&pokemon.id).unwrap().len(), 2);
        repo.rename_custom_tag(tag.id, "Retain").unwrap();
        assert!(repo
            .tags_for(&pokemon.id)
            .unwrap()
            .iter()
            .any(|t| t.name == "Retain"));
        repo.remove_tag(&pokemon.id, standard).unwrap();
        assert!(repo
            .connection
            .execute(
                "INSERT INTO pokemon_tags (pokemon_id, tag_id) VALUES (?1, ?2)",
                params![pokemon.id.0, tag.id.0]
            )
            .is_err());
        assert!(repo
            .connection
            .execute(
                "INSERT INTO pokemon_tags (pokemon_id, tag_id) VALUES ('missing', ?1)",
                [tag.id.0]
            )
            .is_err());
        assert!(repo
            .connection
            .execute(
                "INSERT INTO pokemon_tags (pokemon_id, tag_id) VALUES (?1, -1)",
                [&pokemon.id.0]
            )
            .is_err());
        assert!(matches!(
            repo.assign_tag(&pokemon.id, TagId(-1)),
            Err(Error::NotFound("Tag"))
        ));
        repo.remove_tag(&pokemon.id, tag.id).unwrap();
        repo.remove_tag(&pokemon.id, tag.id).unwrap();
        assert!(repo.tags_for(&pokemon.id).unwrap().is_empty());
        repo.assign_tag(&pokemon.id, tag.id).unwrap();
        repo.delete_custom_tag(tag.id).unwrap();
        assert!(repo.tags_for(&pokemon.id).unwrap().is_empty());
        let standard = repo.tags().unwrap()[0].id;
        repo.assign_tag(&pokemon.id, standard).unwrap();
        repo.delete(&pokemon.id).unwrap();
        assert_eq!(
            repo.connection
                .query_row("SELECT count(*) FROM pokemon_tags", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn reopening_database_preserves_identity_and_assignments() {
        let path = std::env::temp_dir().join(format!(
            "tcm-inventory-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let repo = InventoryRepository::open(&path).unwrap();
        let mut data = input("Buddy");
        data.ivs = Some(AppraisalIvs {
            attack: 15,
            defense: 14,
            hp: 13,
        });
        let pokemon = repo.create(&data).unwrap();
        let tag = repo
            .tags()
            .unwrap()
            .into_iter()
            .find(|t| t.name == "SAVE/SPECIAL")
            .unwrap();
        repo.assign_tag(&pokemon.id, tag.id).unwrap();
        drop(repo);
        let repo = InventoryRepository::open(&path).unwrap();
        assert_eq!(repo.get(&pokemon.id).unwrap(), pokemon);
        assert_eq!(repo.tags_for(&pokemon.id).unwrap(), vec![tag]);
        assert!(repo
            .connection
            .execute(
                "INSERT INTO pokemon_tags (pokemon_id, tag_id) VALUES ('missing', 1)",
                []
            )
            .is_err());
        drop(repo);
        std::fs::remove_file(path).unwrap();
    }
}
