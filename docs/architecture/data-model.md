# Data model

## Implemented schema

`schema_migrations` records applied immutable migration versions. Version 1 creates `application_settings` for local metadata; version 2 adds inventory and seeds standard tags; version 3 adds appraisal IVs and the `SAVE/SPECIAL` standard tag. Versions 1 and 2 are unchanged. Pending migrations run in order in one transaction and record their versions only on success. Reopening the database does not rerun applied migrations or reseed tags.

| Table          | Fields and constraints                                                                                                                                                                                                                            |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `pokemon`      | `id`: canonical lowercase UUID v4 primary key; `display_name`: required manual label; `cp`: optional positive SQLite integer; `iv_attack`, `iv_defense`, `iv_hp`: optional appraisal group, each 0–15; `created_at`, `updated_at`: UTC timestamps |
| `tags`         | Integer primary key; `name`: required unique label using SQLite `NOCASE`; `scope`: `standard` or `local`; `created_at`: UTC timestamp                                                                                                             |
| `pokemon_tags` | Composite primary key (`pokemon_id`, `tag_id`); foreign keys with delete cascade; `created_at`: UTC timestamp                                                                                                                                     |

The assignment primary key supports lookup by Pokémon; a reverse `(tag_id, pokemon_id)` index supports tag lookup and cascade cleanup. No speculative indexes or reference datasets are included.

## Validation and ownership

- The Rust repository generates UUID v4 identities from SQLite's native random bytes, setting version and variant bits. Identity never depends on a label, species, import order or row position. UUIDs are identifiers, not secrets. A primary key rejects collisions; errors are returned without overwriting an existing record. Update input contains no identity field; a database trigger also prohibits changing the UUID.
- A manual display name is not a species identifier. Names contain 1–100 Unicode scalar values; tag labels contain 1–50. Rust rejects leading/trailing Unicode whitespace and control characters. Inputs are rejected rather than silently normalized. SQL adds basic length/trim checks as a second boundary.
- CP is unknown (`NULL`) or a positive signed 64-bit integer. No game-specific maximum or inferred species/level relationship is assumed.
- Appraisal IVs are either all unknown (`NULL`) or all present, each an integer from 0 to 15. `iv_hp` is the appraisal HP IV, not current or maximum observed HP. Rust represents the group as `Option<AppraisalIvs>` and validates each value. SQLite range checks and a cross-column `CHECK` on the last added column enforce the same rules on inserts and updates, including direct SQL. Existing version 2 records retain their UUIDs, names, CP, timestamps and tag assignments; their IVs start unknown. Updating with `ivs: None` explicitly clears all three IVs.
- Every instance has zero or more tags; duplicate assignments cannot exist. Assignment and removal are idempotent for existing entities. Missing Pokémon/tags produce typed not-found errors. Deleting an instance or custom tag cascades only its assignments.
- Standard tags are `EVOLVE`, `POWER UP`, `ATTACKER`, `GYM DEF`, `ROCKET`, `PVP 1500`, `PVP 2500`, `TRANSFER`, `SAVE/SPECIAL`. They are migration-owned. `SAVE/SPECIAL` expresses the user's intent to retain a special Pokémon (for example, a shiny); it does not assert or automatically detect shiny status. Repository operations and database triggers prevent renaming, deletion, promotion of custom tags, or insertion of extra standard tags. Future migrations changing the standard set must explicitly revise these triggers.
- Migration 3 temporarily replaces the standard insert/update guards inside the migration transaction to seed `SAVE/SPECIAL`. If a custom tag already uses that name (ASCII case-insensitively), it is promoted to standard with canonical spelling, preserving its identifier, creation timestamp and assignments. Other custom tags are unchanged. The guards are restored before commit; a failure rolls the migration back.
- Custom tags belong to this local application database (`scope = local`), with no account or remote ownership. Names share one unique namespace with standard tags. SQLite `NOCASE` folds ASCII case only: `Favorites` and `FAVORITES` collide; Unicode case variants and differently normalized Unicode strings remain distinct. Custom tags can be renamed and deleted, but cannot claim a reserved standard name.
- Timestamps use SQLite `CURRENT_TIMESTAMP` (UTC, second precision). Creation timestamps are preserved by repository updates; Pokémon `updated_at` is set on each update, and can equal its prior value within a second. Custom tags currently record creation only. Concurrent edits use SQLite serialization and last successful write wins; no edit-history or optimistic-lock contract exists yet.

## Native persistence boundary

`InventoryRepository::open` initializes/migrates the local database and enables foreign keys/WAL on its own connection. Production callers must use Tauri's resolved per-user app-data path. The path is a native API parameter only, never accepted from the renderer. Startup opens that same repository for the Inventory UI behind typed commands and serialized native access.

Typed APIs provide `create`, `get`, `list`, `update`, `delete`, `tags`, `create_custom_tag`, `rename_custom_tag`, `delete_custom_tag`, `assign_tag`, `remove_tag`, and `tags_for`. `PokemonId` validates canonical UUID v4 strings; `PokemonInput` contains only editable fields, including `ivs: Option<AppraisalIvs>` with `attack`, `defense` and `hp`. Errors distinguish invalid input, missing records, duplicate tag names, immutable standard tags and storage failures. Listing uses creation time then UUID for deterministic order; it currently returns all records without pagination.

All queries and bound parameters remain in Rust. No arbitrary SQL, SQL plugin or generic database Tauri command is exposed. Task 003 adds three narrow review commands; `create_with_tags` atomically creates a record and its assignments and `snapshot` reads records and tags consistently. CP and tag IDs cross IPC as decimal strings to preserve the full SQLite integer range. See [manual review](../development/screenshot-review.md) for native contracts. There are no image blobs, external identifiers, account fields or network calls.

## Evaluation workflow presentation

All current records are presented as awaiting evaluation. No evaluation engine, completed outcome or evaluated transition exists yet, so this is a presentation state for the entire current inventory, not a stored tag or column. Import review sends no tag assignments; previously stored tags and the native tagging contract remain unchanged. Future evaluation must define durable outcomes, versions and state transitions before evaluated records can be distinguished.

## Deferred fields and derived data

Species/form identifiers, species base stats, observed HP, shiny status, second-attack state, fast/charged moves, level, capture details, provenance and import validation state remain unimplemented until their definitions and validation rules are reviewed. Appraisal IVs are separate from species base stats, moves and observed CP/HP. This layer stores observed CP and appraisal IVs without calculating their relationship. Second attacks must be explicit structured fields if introduced, never inferred from tags. Imported screenshot data must be user-reviewed before persistence; original image files must not be retained by default.

Recommendations, rankings and teams are separate future derived data, not properties inferred or stored by this schema. Formula and game-data versions must accompany future derived results. No unversioned formulas, game datasets, reference data, OCR, AI, cloud or telemetry are implemented here. Manual screenshot review and confirmed structured import are implemented without image persistence.
