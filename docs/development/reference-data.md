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

## Future update procedure

1. Establish dataset-specific redistribution rights and factual provenance; retain evidence and required notices. Review licenses independently of this project's MIT code.
2. Build a narrow adapter from a pinned upstream revision. Normalize species/form/individual-variant semantics, single types, GO Stamina and evolution completeness. Do not silently derive missing facts.
3. Publish a reviewed snapshot with a new immutable dataset version, retrieval date and upstream revision; record its content digest in the future distribution manifest. Reusing the current ID/version with different content and returning to a previously active ID/version in the same session are rejected. An in-session rollback requires a fresh version. Cross-release version integrity is enforced by release review/Git, not a persistent registry.
4. Validate with the native loader, source-backed real-species tests and compatibility checks. Preserve stable IDs across versions or design an explicit mapping migration. Deliver through a reviewed app release until a separately designed durable local update workflow exists.

Task 007 should secure dataset rights and validate a bounded real-species adapter/snapshot, including provenance completeness and identity continuity across updates. Moves, level/CP mechanics, battle simulations, meta rankings, teams and investment optimization require later scoped work.
