-- Frozen Task 002 schema, used to verify upgrades of existing databases.
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY NOT NULL,
    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
) STRICT;
INSERT INTO schema_migrations (version) VALUES (1), (2);
CREATE TABLE application_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
) STRICT;
CREATE TABLE pokemon (
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
CREATE INDEX pokemon_tags_by_tag ON pokemon_tags(tag_id, pokemon_id);
