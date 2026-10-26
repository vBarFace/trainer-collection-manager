# 0005: Persisted appraisal metadata and deterministic rules

Date: 2026-10-26. Scope: Task 005, the first reliable evaluation workflow.

## Decision

Keep appraisal rules in a pure Rust module with explicit version `appraisal-1`. Evaluate confirmed attack/defense/HP IVs only. Expose typed descriptive results, explainable findings, missing data and limitations. Keep a narrow future recommendation contract, but emit no strategic recommendations or suggested tags.

Migration 4 adds `pokemon.appraisal_revision` and `pokemon_appraisals`, which stores the Pokémon UUID, evaluated input revision, ruleset version and UTC timestamp for the latest attempt. A null-safe SQLite trigger increments the revision on actual IV changes. Preserve migrations 1–3, existing identities, records and assignments.

Derive Pending, Evaluated, Needs data and Stale in Rust from metadata, current inputs and the supported ruleset. Persist Needs data attempts as well as completed appraisals. Replace metadata atomically on explicit re-evaluation. Extend the existing narrow command ACL with `evaluate_pokemon`; retain native ownership of storage and validation.

## Alternatives and trade-offs

- Timestamps alone cannot reliably invalidate same-second edits. A revision counter detects IV changes and change/revert sequences without a hashing dependency.
- Persisting totals, percentages and findings would duplicate deterministic information. Recalculate current results and store only provenance metadata.
- Keeping old input snapshots or evaluation history would enable historical result reconstruction but adds unnecessary storage/contracts. This increment retains only the latest attempt and withholds stale details until re-evaluation.
- A broad multi-scope evaluation framework and recommendation tables would have no supported strategic data yet. The current schema is deliberately appraisal-specific; future scopes must explicitly define inputs, versions and validity.

Name, CP and tag changes do not invalidate appraisal because they are not inputs to these rules. Appraisal completion must never imply completion of strategic analysis. A rules change must bump the version, making previous attempts stale even after restart.

## Boundaries and future work

Evaluation never changes assigned tags, infers shiny status or user preferences, or recommends transfer/evolution/power-up from CP or IV percentage. Perfect IVs produce an appraisal finding only. Missing IVs remain explicit unknowns.

No new dependency, artwork, model or external dataset is needed. Future strategic support requires validated species/form identification, applicable structured observations, licensed/versioned game data and validated formulas, with user goals/resources where relevant. Screenshot import and temporary OCR retain their existing privacy boundaries. See [evaluation workflow](../development/evaluation.md), [data model](../architecture/data-model.md) and [validation](../development/validation.md).
