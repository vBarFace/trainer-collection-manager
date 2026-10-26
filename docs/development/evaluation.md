# Appraisal evaluation

Task 005 introduces an offline, deterministic appraisal workflow. It evaluates confirmed structured IVs only; it does not run screenshot extraction or read temporary review suggestions.

## Using Inventory

1. Confirm a screenshot review to save a new pending record, or select an existing record.
2. Review its appraisal state, confirmed IVs and assigned tags.
3. Request an appraisal evaluation. The UI shows progress and offers retry on failure.
4. Read the resulting observed facts, findings, missing data and limitations. Re-evaluate after an IV or ruleset change.

The four states are **Pending**, **Evaluated**, **Needs data** and **Stale**. They apply to appraisal only. Evaluation metadata survives reopening the app; tags do not determine state. There is no full saved-record editing UI in this increment. The existing native update API invalidates appraisal when IVs change.

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
