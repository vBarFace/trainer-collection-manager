# 0006: Versioned local reference snapshots and confirmed identity

Date: 2026-10-26. Scope: Task 006, building on merged Task 005 (`0b7b99b`, PR #6) and IV correction (`1169509`, PR #7), both present in base `cca38c9`.

## Decision

Use a native-owned, strictly validated JSON snapshot for reference data, separate from the player's SQLite database. Ship an original empty manifest (`tcm-empty` / `empty-1`) because reviewed sources did not establish sufficient redistribution rights for the underlying GO dataset. Original fictional test fixtures exercise the complete contract without asserting real game facts. See [source research](../development/reference-data-sources.md).

Separate species, GO forms and inventory individuals. Species own stable IDs, National Dex numbers, names and optional aliases (an empty alias list is allowed). Forms own stable IDs, a species ID, names/aliases, nullable one/two types, nullable GO Attack/Defense/Stamina and nullable evolution edges/requirements. Never substitute main-series stats. Shadow, Purified, Shiny and Lucky are individual properties, not generic form records; these properties remain deferred. Regional and Mega form normalization must be reviewed in an upstream adapter. A costume is only a distinct reference form if verified data justifies that distinction.

Migration 5 creates only `pokemon_reference_identity`. Explicitly confirmed species/form queries are retained verbatim, separate from nicknames and tentative OCR. Resolved confirmations also pin the dataset namespace and stable species/form IDs. Names/aliases can change between snapshots without rebinding pinned records. Removed identifiers remain unresolved; a different provider namespace is unavailable until explicit reconfirmation or a future reviewed mapping. No inventory reconstruction or rewrite is needed for catalog updates. Unresolved queries can resolve against a later snapshot, but only queries already explicitly confirmed by the user are eligible.

Reference context reaches the pure Rust evaluation boundary through `assess(ivs, resolution)`, returning the existing appraisal and a separate typed reference context/missing fields. Only the appraisal is persisted as before. `appraisal-1` is unchanged: reference metadata is not a mathematical input to IV totals, so catalog/identity changes must not invalidate appraisal. Future strategic scopes need their own input revision, dataset version and ruleset provenance before persistence.

## Alternatives and trade-offs

- SQLite reference tables would add imports, transactions and joins for a dataset which cannot yet be shipped. JSON keeps the legal boundary and provenance inspectable, can be embedded offline and supports validation before activation. Loading the entire bounded snapshot uses more memory; linear resolution is sufficient at this scale. Add indexes or tables only after measuring a real licensed dataset.
- Inferring identity from the display name would turn nicknames or OCR into false game facts. A narrow saved-record identity editor is explicit and supports correction even while the catalog is empty. No generic full-record edit command is introduced.
- Storing only name queries could silently rebind identities after upstream renames. Store stable IDs when a confirmation resolves, without overwriting the original queries. Do not store derived stats in player rows.
- An online updater and user file-import UI need more provenance, authorization and durable activation design. They are deferred. Native-only `activate_reference` demonstrates whole-snapshot validation and safe replacement; production startup loads the reviewed bundled manifest on every launch.

## Constraints

Schema version 1 rejects unknown fields, malformed JSON, unsupported schema versions, duplicate species IDs/Dex numbers and form IDs, invalid links/types/stats, duplicate evolution targets and evolution cycles. All provenance fields must be present and nonempty; nullable metadata must be explicitly supplied. A provenance declaration is not proof of rights or factual correctness: release review must verify both.

Versions are opaque strings. Reusing the current dataset/version with different content is rejected; idempotent activation is accepted. A small in-memory set prevents returning to a previously active dataset/version, including A → B → modified A; rollback in that session requires a fresh version. The implementation does not persist that set or activated snapshots. A future distributor must pin upstream revisions and content digests and enforce immutable versions across releases. Failed activation preserves the active snapshot and all player data.

The only new IPC command confirms identity with an expected/current value comparison under a SQLite immediate transaction. It accepts neither file paths, datasets, SQL nor observations. Main-window ACL and existing offline/privacy boundaries remain intact.

## Task 007 refinement — 2026-10-26

The empty production catalog, schema 1 and confirmation/pin contracts remain unchanged. A development-only local-file PvPoke adapter uses explicit reviewed mappings and validates complete candidates through `Catalog::load` in a feature-gated CLI. Source/plan/output SHA-256 digests and a cumulative release ledger accompany candidates; no network code or new dependencies are introduced. Rights remain unverified, so only original fictional fixtures are committed.

Activation now rejects a previously seen species ID acquiring another National Dex number, a Dex number acquiring another species ID, or a form ID changing species within a dataset namespace. Session history survives temporary removals and provider switches. This narrowly closes an ID-reuse path that could otherwise change the meaning of existing pins while retaining version-validation behavior. Cross-release history belongs in reviewed provenance, rather than a new player database schema; semantic changes retaining Dex/parent still require human evidence review. See the [adapter and release workflow](../development/reference-data.md#task-007-adapter-and-release-workflow).
