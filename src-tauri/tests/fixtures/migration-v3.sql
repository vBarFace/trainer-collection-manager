-- Frozen Task 003 upgrade; apply after schema-v2.sql to model Task 004 databases.
ALTER TABLE pokemon ADD COLUMN iv_attack INTEGER CHECK (iv_attack BETWEEN 0 AND 15);
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
    BEGIN SELECT RAISE(ABORT, 'Standard tags are migration-owned'); END;
INSERT INTO schema_migrations (version) VALUES (3);
