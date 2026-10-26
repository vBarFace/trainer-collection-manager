# Appraisal evaluation

Task 005 introduces an offline, deterministic appraisal workflow. It evaluates confirmed structured IVs only; it does not run screenshot extraction or read temporary review suggestions.

## Using Inventory

1. Confirm a screenshot review to save a new pending record, or select an existing record.
2. Review its appraisal state, confirmed IVs and assigned tags.
3. Request an appraisal evaluation. The UI shows progress and offers retry on failure.
4. Read the resulting observed facts, findings, missing data and limitations. Re-evaluate after an IV or ruleset change.

The four states are **Pending**, **Evaluated**, **Needs data** and **Stale**. They apply to appraisal only. Evaluation metadata survives reopening the app; tags do not determine state. Saved appraisal IVs can be corrected as described below; full saved-record editing remains deferred.

## Correcting saved IVs

Select a record, then choose **Correct saved IVs**. Review the prefilled attack/defense/HP group and enter all three whole numbers from 0 to 15. To clear known values, choose **Mark all IVs unknown** or leave all three empty. Both are draft changes until **Confirm IV correction** is submitted. Partial groups are rejected. **Cancel IV correction** and switching to another record discard the unsaved draft.

Saving changes only IVs, preserving the record's UUID, name, CP, creation timestamp and tags. Actual changes update the record timestamp and make an existing appraisal attempt Stale through the existing migration-4 trigger. Unchanged values are a no-op, preserving timestamps and state. A record without an appraisal attempt remains Pending. Re-evaluate explicitly after correction; saving never runs appraisal or assigns tags.

The form disables duplicate submission, record selection, evaluation and cancellation while saving. Ordinary failures retain the draft for retry. If saved IVs changed while the draft was open, the native command rejects the save; discard the draft and reload, or cancel. A deleted record likewise requires reload. Drafts stay in memory and are not recovered after leaving the screen or closing the app. No screenshot is required.

`correct_pokemon_ivs` accepts a validated Pokémon UUID and `{ expectedIvs, ivs }`, with both groups explicitly supplied as `null` or complete integer groups. Rust validates both groups and compares current IV values with `expectedIvs` in an immediate write transaction before changing only IV columns. This is a value comparison, not a revision-token contract: an intervening change/revert that restores the expected group is accepted. No new migration, dependency or appraisal ruleset is needed.

## Ruleset `appraisal-1`

The repository accepts either a complete attack/defense/HP IV group or an unknown group. Each known value is 0–15. The engine reports those observed IVs, their total (0–45), and descriptive appraisal percentage (`total × 100 / 45`, displayed to two decimal places). A 15/15/15 group produces a perfect-appraisal finding. An unknown group produces explicit missing attack/defense/HP data instead of a percentage or guessed values.

The percentage describes appraisal only. It is not a PvP ranking, PvE strength score, overall quality score or transfer criterion. A perfect appraisal does not establish universal competitive superiority. Species, moves, level, game meta, shiny status and user preferences are not inferred. No transfer, evolve, power-up or other strategic recommendation is emitted, and no suggested tag is applied. Findings, future recommendations and already assigned tags remain distinct.

## Persistence and invalidation

Migration 4 adds a native-owned IV revision and one appraisal metadata row per evaluated Pokémon. The row records evaluated revision, ruleset version and UTC timestamp, including Needs data attempts. Lifecycle state and descriptive results are recalculated in Rust. No screenshot, path, raw OCR text, draft extraction, duplicated result or old-input snapshot is stored.

Actual IV changes increment the revision through a null-safe SQLite trigger. Name, CP and tag changes are outside this scope. Revision or ruleset mismatch makes the previous attempt Stale, including a previous Needs data attempt. Stale results are withheld until re-evaluation; their prior timestamp/version remain available. Re-evaluation replaces the latest attempt atomically. There is no evaluation history. Deleting a Pokémon cascades its appraisal metadata.

`evaluate_pokemon` accepts a canonical lowercase UUID v4 through the existing typed Tauri boundary and local main-window ACL. Rust owns validation, SQL and business rules. Invalid IDs and missing/deleted Pokémon fail safely. No frontend SQL or external game data is involved.

## Prerequisites for strategic evaluation

Future PvP, PvE, gym, Rocket, investment and transfer support requires its own scope and ruleset contract. Prerequisites include validated species/form identification, relevant confirmed moves/level and other observations, reviewed licenses and provenance for versioned game datasets, validated formulas and explicit user goals/resources where applicable. CP or aggregate IV percentage alone cannot justify transfer, evolution or power-up. Recommendation codes, actions/classifications, supporting facts, explanations, versions and limitations must remain separate from assigned tags.

The current increment adds no dependency, game dataset or third-party asset. Representative OCR accuracy measurement and wider layout support remain separate work; the existing single supplied screenshot does not establish recognition accuracy. See [data model](../architecture/data-model.md), [decision 0005](../decisions/0005-persisted-appraisal-evaluation.md) and [validation results](validation.md).
