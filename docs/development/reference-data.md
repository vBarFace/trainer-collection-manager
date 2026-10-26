# Local reference data

Task 006 provides a validated, offline reference foundation. **This release bundles no real Pokémon GO records.** The UI can retain manually confirmed species/form queries, but real game metadata remains unavailable. This is the compliant fallback allowed by the task, not a completed GO encyclopedia. See [source and licensing research](reference-data-sources.md) and [decision 0006](../decisions/0006-versioned-local-reference-data.md).

## Ownership and schema

`src-tauri/reference/catalog.json` is compiled into the native application. `reference::Catalog::load` parses at most 16 MiB and validates the complete candidate before returning a usable catalog. The serialized envelope contains:

- `schemaVersion`: currently exactly `1`.
- `dataset`: `datasetId`, opaque `version`, HTTPS `sourceUrl`, pinned `upstreamRevision`, `license`, `notice`, `retrievedAt` and HTTPS `rightsEvidenceUrl`. URL strings are provenance only: the application never fetches them.
- `species`: unique stable lowercase IDs, unique positive `dexNumber`, canonical `name` and `aliases`. Aliases can include localized spellings, but this is not a localization service.
- `forms`: globally unique IDs, existing `speciesId`, `name`, `aliases`, explicit nullable `types`, `baseStats` and `evolutions`. Types are one/two distinct values from the 18 supported types. GO stats are positive integers (`attack`, `defense`, `stamina`), not appraisal IVs or observed HP. Unknown grouped stats are `null`; partial groups are rejected.

An evolution edge has an existing `targetFormId` and explicit nullable `requirements`. Requirements are verified descriptive strings at this stage, not machine-actionable costs or eligibility rules. `evolutions: null` means unknown; `[]` means the dataset records no evolution. Requirements `null` means unknown; `[]` means none recorded. Adapters must not turn incomplete upstream lists into verified empty lists. Dangling/duplicate edges and cycles are rejected. Form changes which are not evolution, such as reversible Mega transformations, must not be encoded as cyclic evolution edges.

Species/form IDs are stable only within a dataset namespace. Catalog validation checks structure and consistency; it cannot verify real game facts, license authenticity, upstream completeness or identifier stability. No artwork, sprites or downloaded datasets are included. `serde_json`, already present as a test/transitive dependency under MIT/Apache-2.0, becomes a direct runtime dependency for native snapshot parsing; no new package is added.

## Resolution and inventory

Select an inventory record and choose **Correct species and form**. Confirm an exact stable identifier, canonical name or alias. Species and form are separate from the display name; OCR/import review never sets identity. Leave form empty when unknown; clear both fields to remove identity. Unsupported text is retained verbatim so it can be corrected later. No fuzzy matching, accent stripping, punctuation substitution or implicit default form is applied. Names/aliases match using Unicode lowercase comparison. Exact stable IDs take precedence; multiple name/alias matches return ambiguity.

States are `not_set`, `unavailable`, `unknown_species`, `unknown_form`, `ambiguous` and `resolved`. Missing form is unresolved even if only one form is in the snapshot. Partially known species may be displayed without claiming a known form. Empty catalog and mismatched pinned provider namespaces are unavailable. The production empty catalog is identified and explained visibly.

Migration 5 adds a separate `pokemon_reference_identity` table with UUID foreign key/delete cascade, `species_query`, nullable `form_query`, grouped nullable `dataset_id`/`species_id`/`form_id` and `confirmed_at`. A missing row means identity unknown. The grouped IDs are present only when a confirmation resolves. They are not foreign keys into the immutable external snapshot. On reading a pinned association, use IDs rather than reinterpreting queries. If an ID disappears, preserve the queries and pin while exposing unresolved state.

`correct_pokemon_identity(id, {expectedIdentity, identity})` requires both nullable groups explicitly. It validates text, compares expected identity against the saved identity under an immediate write transaction, then atomically replaces/removes the association and returns the record. Stale drafts cannot overwrite different confirmed text. Comparison is value-based; change/revert is accepted. Reconfirmation of unchanged text preserves an existing pin, even when an alias was removed or a different catalog is active. An unpinned query can acquire a pin on confirmation after a supported snapshot becomes available. Unknown data remains valid manual input. Names, CP, IVs, UUIDs, timestamps, tags and appraisal attempts are untouched. Existing migrations 1–4 remain immutable.

## Evaluation and offline behavior

`evaluation::assess` accepts confirmed IVs and a resolved/unresolved reference value without a repository or UI dependency. It returns descriptive appraisal plus typed reference context and missing fields (identity, types, base stats, evolutions). Missing reference data never prevents valid IV appraisal. No strategic formula, ranking, action recommendation, automatic tag or simulation is added. The existing IPC appraisal contract and `appraisal-1` lifecycle remain unchanged; current reference metadata is separately exposed on `RecordView.reference`.

Startup validates the bundled manifest before exposing Inventory. An invalid bundled manifest prevents repository opening with a safe error rather than activating bad data. Test/future native callers can use `activate_reference` to validate a candidate before replacing the in-memory snapshot. Invalid candidates preserve the active snapshot. Updates do not write player data. Production has no import picker, network client or automatic update service, and an in-memory replacement is not durable across restart.

## Task 007 adapter and release workflow

Task 007 implements `scripts/reference-adapter.mjs`, an offline, dependency-free Node development tool for PvPoke's `src/data/gamemaster/pokemon.json` shape. The inspected technical baseline is full SHA `38f70320d190e38c2e6c778f572662b4c4e8185d`. **No real-data candidate is authorized or bundled.** The original fictional source/plan under `scripts/fixtures/reference/` exercise two invented species and four forms: standard, coastal, Mega and incomplete metadata. Their numbers, names, relationships and stats assert no real game facts. One additional fictional Shadow entry demonstrates subset exclusion. Fictional fixtures never participate in runtime startup.

### Reproduce the fictional candidate

Use Node 24 and the existing native development prerequisites. Run from repository root; create the ignored working directory first. Choose fresh output directories for every run:

```powershell
New-Item -ItemType Directory -Force reference-work
npm run reference:generate -- --source scripts/fixtures/reference/source.json --plan scripts/fixtures/reference/plan.json --out reference-work/fiction-1
npm run reference:generate -- --source scripts/fixtures/reference/source.json --plan scripts/fixtures/reference/plan.json --out reference-work/repeat --previous reference-work/fiction-1/provenance.json
npm run reference:test
npm run reference:test-native
```

Each output directory contains a validated `catalog.json` and `provenance.json`. The tool rejects existing output directories and the bundled reference directory. It does not activate or install candidates. It invokes the feature-gated development binary with `cargo run --quiet --manifest-path src-tauri/Cargo.toml --features reference-tools --bin validate-reference`, passing the entire candidate through stdin to the same `Catalog::load` used by production. No database, app-data directory, IPC capability or network client is involved. The validator is excluded from the default application build. A failed native validation writes no output; an OS write failure can leave an incomplete new output directory, which must not be released.

`npm run check` includes the portable Node transformation/provenance suite. `reference:test-native` separately requires the Windows/native toolchain and runs whole-candidate rejection plus CLI/digest/repeat/no-overwrite checks. CI runs the portable tests in the frontend job and Rust/native integration tests in the Windows job.

### Input and semantic review contract

The CLI requires `--source LOCAL_FILE --plan REVIEWED_PLAN --out NEW_DIRECTORY`; `--previous PROVENANCE_FILE` carries release history forward. Source, plan and previous provenance inputs are each bounded to 16 MiB. There is deliberately no downloader. For a real release, obtain the exact file from a clean checkout at the approved full Git SHA, preserve original bytes and compare it with that pinned Git object during review. A local file's claimed source/revision is not authenticated by a digest alone. Never use mutable `master`/`latest` as a release input.

Use the fictional plan as the documented schema example:

- `adapterVersion` is exactly `pvpoke-1`. `dataset` uses existing schema-1 metadata. Real review status requires a full lowercase 40-character commit SHA and its exact raw PvPoke file URL; fictional inputs require an `original-...` revision and `synthetic-` namespace. `retrievedAt` is an explicitly supplied valid calendar date. All metadata is supplied, never read from the clock.
- `rights` has `status`, `review` and `evidenceUrl` matching dataset rights evidence. `fictional` means original invented fixtures; `reviewed` means a maintainer has independently verified dataset-specific redistribution authority and provenance. Any other status blocks generation. A user-provided status is an assertion, not legal verification; release review must examine the evidence. The repository includes only the fictional plan.
- `species` maps stable IDs, National Dex numbers, canonical names and distinct verified aliases, with evidence URLs. `forms` explicitly maps each selected `upstreamId` to stable form/species IDs, canonical name, aliases, `kind` (`standard`, `regional`, `other`, `mega`), evidence URL and evolution knowledge. No names, aliases or default forms are inferred from upstream strings. Dex must agree with the mapped species. Unselected entries are intentionally outside bounded coverage; duplicate upstream IDs are rejected globally. Missing selected IDs fail.
- Selected rows must have the reviewed required fields and supported field names. Extra upstream fields require adapter review; known unused simulation fields are ignored. New unused fields on unselected rows do not affect the selected subset. `baseStats` must be explicit `null` or the complete positive integer `atk`/`def`/`hp` group; HP is mapped to GO Stamina under the verified baseline semantics. Main-series stats, CP and IV-derived values are never accepted as replacements. `types` must be explicit `null` or one/two supported distinct values; only a second-slot `none` placeholder is removed. Unsupported type strings fail instead of being converted to unknown.
- Shadow, Purified, Shiny and Lucky tags/ID tokens are rejected on selected records. Those are individual attributes. Mega tags require explicit Mega mapping; Mega forms have unknown evolutions and cannot be evolution sources or targets. Other reversible form changes must be reviewed and left out of evolution groups. `family`/`formChange`/nicknames do not supply verified relationships or aliases.
- `evolutions: null` is unknown. A known group must explicitly provide `evidenceUrl` and complete `edges`; `edges: []` asserts none recorded and requires evidence just as populated groups do. Each edge supplies `targetFormId` and explicit nullable `requirements`. Empty requirements remain distinct from unknown. Costs/conditions are reviewed descriptive strings only. The source's family links are never used as proof of direct eligibility or completeness. Targets must be included in the bounded candidate; native validation rejects dangling/duplicate edges and cycles.

Object keys, species/forms, alias lists and evolution target/requirement lists are deterministically sorted using ordinal comparison rather than locale. Type order is preserved as primary/secondary. Source record and review-plan collection ordering do not change snapshot bytes. Output uses UTF-8, two-space indentation and a final LF. Source/plan byte digests intentionally change if those inputs are reformatted, even when snapshot content does not. No dependency was added.

### Provenance, compatibility and release

`provenance.json` records adapter version, dataset namespace, source URL, upstream SHA/original revision, retrieval date, rights review and SHA-256 of exact source bytes, exact plan bytes and exact emitted catalog bytes. Keep the plan and previous release provenance under review alongside any future authorized snapshot and notices. The cumulative `identityLedger` retains species ID↔Dex, form ID→species/upstream ID/kind and version→output digest, including removed identities. Updates must pass the previous reviewed provenance via `--previous`; omission starts a new ledger and is allowed only for the first release of a namespace. The ledger is trusted release input, not signed evidence: review its integrity/history in Git and compare recorded output digests to released files. Syntax/conflicts are checked; the tool cannot prove ownership, facts or an honest prior ledger.

Renames, alias changes, added records, removals and explicit later `null` metadata are compatible. Reusing IDs/Dex associations, remapping an upstream form or changing a form's kind fails; changing content under an already recorded dataset version fails. Release rollback uses a new version. Keep tombstones through removals; never reset the ledger to bypass a conflict. A breaking upstream identity change requires a separate explicit mapping proposal listing old/new dataset/species/form IDs, source evidence, affected confirmations and unresolved cases, with migration tests and user-facing reconfirmation behavior before implementation. No mapping or player-data migration is performed here.

Native activation additionally keeps session history per dataset namespace for species ID↔Dex and form ID→species, including removed entries and provider switches. An inconsistent candidate is rejected before replacing the catalog. Existing pins/queries, observations, UUIDs, CP, IVs, tags, timestamps and appraisal attempts remain intact. This history is not persisted: cross-release continuity depends on the reviewed cumulative ledger and release process. A semantic reuse retaining the same Dex/parent cannot be discovered structurally; explicit mapping evidence/review remains required. No schema change was needed.

For an authorized release: establish rights first, pin/verify source bytes, review every bounded identity/metadata assertion, supply a fresh immutable dataset version and previous ledger, generate and run all checks, independently compare provenance digests, retain required notices, then review the candidate before replacing the bundled catalog through an app release. No production updater exists. Today the licensing gate remains blocked, the production `tcm-empty` / `empty-1` is unchanged, and Inventory continues to explain missing reference data while retaining confirmations. See [dated evidence](reference-data-sources.md#task-007-evidence-review--2026-10-26) and [validation results](validation.md).

Moves, level/CP mechanics, battle simulations, meta rankings, teams and investment optimization require later scoped work.
