# Development roadmap

1. Implemented (Task 002): local inventory schema, stable UUIDs, validation, grouped optional appraisal IVs, standard/custom tags and typed Rust persistence.
2. Implemented (Task 003): native PNG/JPEG selection, bounded temporary preview, manual review, explicit atomic confirmed saving and a minimal persistent **Awaiting evaluation** Inventory list. Import review confirms data without selecting classification tags; previously saved tags remain intact. OCR and automatic extraction are deferred.
3. Implemented bounded increment (Task 004): typed per-field extraction, conservative experimental appraisal-bar geometry, editable tentative suggestions and safe manual fallback. No new dependencies or persistence fields. Name/CP OCR and real-image accuracy validation remain unsupported; see [local extraction](local-extraction.md).
4. Implemented Task 004 follow-up: bundled offline Tesseract OCR for name/CP, native crops and real grey/orange/pink appraisal support. Supplied example verified locally as Abra / CP468 / IV15/2/2; no general accuracy rate is claimed.
5. Implemented (Task 005): persisted appraisal lifecycle, `appraisal-1` deterministic Rust rules and Inventory evaluation/re-evaluation. Results describe confirmed IVs, total, percentage and perfect appraisal, with explicit missing data and limitations. IV revisions and ruleset versions invalidate outdated attempts. Suggested and assigned tags remain separate; no strategic recommendation or tag is generated. See [evaluation](evaluation.md).
6. Implemented follow-up: minimal confirmed saved-IV correction, including complete/unknown groups, cancellation, retained drafts after failure and protection against overwriting different saved IVs. The existing schema and appraisal invalidation are reused; names, CP and tags are unchanged and evaluation remains explicit.
7. Implemented (Task 006): validated/versioned native JSON reference snapshots, explicit confirmed species/form identity, stable association pins, reference context for evaluation and minimal Inventory metadata/correction UI. Migration 5 preserves player observations and appraisal lifecycle. Production uses an empty catalog pending verified dataset redistribution rights; fictional fixtures are test-only. See [reference data](reference-data.md) and [source research](reference-data-sources.md).
8. Merged safe scope (Task 007, PR #9): pinned/local-file PvPoke adapter, explicit reviewed identity/form/evolution mappings, deterministic schema-1 output, native validation, SHA-256 provenance, cumulative release identity/version ledger and cross-version/inventory preservation tests. Confirmed in remote/local main `0a365d145d7cc9c530e28b4314db46285ea4b8d7` during Task 008 inspection; no newer remote main commit was found. The main CI run was initially in progress; final read-only verification confirmed [run 40385059460](https://github.com/vBarFace/trainer-collection-manager/actions/runs/40385059460) completed successfully for that exact SHA. Source rights/provenance evidence was last reviewed in Task 007 on 2026-10-26. **Blocked acceptance criterion: bounded real-species snapshot**, because dataset-specific redistribution authority remains unverified. Production retains `tcm-empty` / `empty-1`; original fictional fixtures exercise the integrated workflow. Next reference milestone: review dataset-specific authorization/provenance, verify a bounded real subset and ship through reviewed release. Independently, build a representative authorized labelled OCR benchmark, extend supported layouts and calibrate confidence.
9. Task 008 documentation scope: specify individual utility assessment, multiple roles, readiness, next-action suggestions, progression roadmaps, progress evidence and optional future hybrid evaluation. See [PRD](../product/PRD.md#utility-classification-and-progression-roadmaps--proposed), [decision 0007](../decisions/0007-individual-progression-roadmaps.md) and the proposed backlog below. No strategic code, training, persistence or UI is delivered by this specification.

Each milestone requires separate product, security and license review.

## Proposed delivery backlog

These are local issue proposals, not created GitHub issues. S1–S7 are small delivery increments; the data/mechanics gate is an independent prerequisite for real strategic release. Within each increment keep changes bounded to the named slice. Full inventory editing, deletion, filtering, teams, generic workflows and conversational AI remain deferred.

### S1 — Lock the first utility contract with fictional scenarios

**Implemented S1 contracts only (2026-10-27):** isolated Rust/TypeScript `utility-1` types, bounded pure native validation and nine shared original fictional acceptance cases. No production command, inventory DTO, planner, persistence, progress command, UI or ML uses them. See [implemented wire contract and limits](utility-contracts.md) and [validation](validation.md). The scope/acceptance description below is retained from Task 008; S2 remains the next delivery increment.

Dependencies: Task 008 specification. **Original planned scope, now delivered.** Define typed native/renderer strategic DTOs and a contract test matrix for one selected goal per UUID: current/potential roles, readiness, nullable next-action/summary tag, short alternative paths, conditional prerequisites and provenance. Keep the contracts disconnected from production evaluation and persistence; use only original fictional cases matching PRD scenarios A–E.

Acceptance: serialization/validation tests distinguish role/action kinds and unknown/zero costs, reject out-of-order prerequisites and mismatched UUIDs, and cover use now, evolve-first, power-up-first, missing data, protection conflict and stale progress. Document no production strategic output, migration, model, new dependency or real game claim. Existing appraisal, tags and empty catalog contracts remain unchanged.

### S2 — Confirm the minimum individual and goal inputs

**Recommended next bounded implementation task:** start with saved CP correction and a separate strategic observation revision. Reuse the narrow expected-value correction pattern; test unknown/positive CP, no-op/conflict handling, revision change/revert, UUID/tag preservation and unchanged `appraisal-1` lifecycle. Do not add a planner or roadmap storage in that follow-up. Remaining S2 inputs can follow in separate reviewed slices.

Dependencies: S1; a field/units contract for the bounded attacker goal. Add only applicable confirmed moves/second-attack state, level/evidence, individual properties, saved CP correction, goal, budget and protections; allow explicit unknowns. Implement native-owned strategic/constraint revisions with expected-value/revision correction and minimal confirmation UI. Any migration is separately reviewed and preserves migrations 1–5.

Acceptance: actual changes, no-ops, same-second change/revert, concurrent drafts, unknown groups and restart behave as specified; resulting evolution observations remain on the same UUID. No stale pre-action values survive as confirmed new observations. Existing UUIDs/tags and independent appraisal invalidation are preserved; no game or screenshot evidence is fabricated.

### Gate D — Authorize and validate the bounded game data/mechanics

Dependencies: Task 007's merged workflow; separate evidence review, without rebuilding its adapter. Resolve the real-species snapshot acceptance criterion and review additional move/combat facts, level/stat rules, evolution eligibility, resource costs, move availability and exceptions required by the chosen goal. Design a versioned typed mechanics contract rather than treating schema-1 prose as rules. Use the [rights matrix](../legal/third-party-assets.md#proposed-strategic-data-and-training-review-task-008).

Acceptance: exact included fields/revisions/digests/notices have recorded authority for their intended uses; bounded factual examples and calculations have independently reviewed expected results; unknowns remain explicit; unsupported mechanics abstain. Compatible IDs and inventory-preservation tests pass. This gate is currently **blocked** on rights and strategic mechanics evidence; no real strategic release can bypass it.

### S3 — One deterministic ATTACKER vertical slice

Dependencies: S1–S2; Gate D before real-data acceptance. Implement a pure Rust planner for one named PvE goal, with current-versus-prepared comparison, hard budget/protection checks, justified target, required moves, ordered steps and use-now alternative. Return transient typed results and a minimal explicit assessment preview; defer plan persistence/progress controls to S4.

Acceptance: reviewed examples match expected role/readiness/action; no emitted path violates eligibility, move availability, budget or protections; missing costs/mechanics block investment. Evolution target and all dependencies are explicit. Fictional tests can verify the contract before Gate D, but must remain visibly fictional and cannot satisfy real strategic accuracy. Existing appraisal produces no strategic suggestions and assigned tags never change automatically.

### S4 — Persist a roadmap and review progress

Dependencies: S2–S3; approved storage/migration and IPC design from the [data model](../architecture/data-model.md#proposed-strategic-assessments-and-roadmaps-task-008). Add current/previous plan slots, progress evidence, safe reload/re-evaluation and accessible progress UI. A fictional demonstration may precede Gate D if clearly separated from real recommendations.

Acceptance: restart preserves UUID association, input/version context and reports; checkbox completion alone never changes observations or spending. Evolution/power-up/move confirmation follows the PRD minimums. Concurrent progress writes, failed rotation, changed goals/resources and rules/data/model updates preserve data and withhold stale suggestions. Re-evaluation explains changes and requires review before carrying prior progress. Test keyboard flow, safe errors, retry and migration preservation.

### S5 — Add GYM DEF and further feasible alternatives

Dependencies: S3–S4 and separate defense-goal/mechanics evidence under Gate D. Define defense utility with explicit opponent, time and gym assumptions; add only then its deterministic role assessment and goal-specific paths. Preserve other roles as unsupported rather than silently evaluating them.

Acceptance: the same individual can have multiple roles with different readiness; defense and attacker targets are not conflated. Required/optional moves, mutually exclusive path costs and no-investment/wait choices remain understandable. Reviewed defense examples and budget/protection failure cases pass before real release.

### S6 — Build reviewed labels and run the first ML experiment

Dependencies: S3 baseline and Gate D rights review extended to training/retention/model redistribution; S5 is optional. Freeze the bounded attacker label rubric, consented/original contribution format, grouped splits and simple baselines. Train/evaluate small role-suitability models offline in development only. Path preferences and other roles require separate labels and later experiments.

Acceptance: dataset card, label disagreements/unknowns, provenance and allowed uses are recorded; family/form and snapshot leakage controls, untouched holdout, uncertainty intervals and comparison with the strongest simple deterministic baseline are reproducible. Report separate suitability/action/plan metrics and resource measurements from the [evaluation plan](evaluation.md#proposed-strategic-and-learning-evaluation-task-008). Poor or inconclusive models are rejected; no automatic screenshot contribution or runtime weights are included.

### S7 — Optional hybrid inference after successful evaluation

Dependencies: S6 evidence meeting predeclared release criteria; S4 lifecycle/provenance; reviewed model distribution rights and offline budget. Integrate only bounded suitability scoring, feature compatibility/digest checks and deterministic validation/fallback. Ranking paths is a separate extension after preference labels exist.

Acceptance: model absence, corrupt/incompatible artifacts, timeout, uncertain score and unsupported inputs fall back or abstain without an invalid action. Confirmed protections and budgets always win. Plan provenance distinguishes deterministic and hybrid output; model updates stale affected plans. Measure cold/warm latency, memory and package size on supported Windows hardware, demonstrate network-free operation and re-run held-out suitability/action/plan gates. No model benefit is claimed without evidence.
