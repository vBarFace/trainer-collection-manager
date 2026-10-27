# Architecture overview

The desktop process uses Tauri 2 and Rust. React + TypeScript renders accessible local inventory review and placeholder navigation. Rust resolves the per-user application-data directory, opens `trainer-collection-manager.sqlite3`, enables foreign keys/WAL and applies ordered migrations.

- `src/Inventory.tsx`: editable data review with tentative extraction provenance and no tag selection, explicit save confirmation, appraisal lifecycle display and record selection for evaluation/re-evaluation. Existing assigned tags remain separate from findings and recommendations.
- `src/AppraisalPanel.tsx`: accessible appraisal details, current inputs/results, ruleset provenance, missing data, limitations and entry to saved-IV correction; no duplicated evaluation rules.
- `src/IvCorrectionForm.tsx`: temporary grouped IV draft, explicit confirmation/cancellation, accessible progress/errors and retained values after recoverable failure. `src/review.ts` shares input-format validation with import review; native validation remains authoritative.
- `src/native.ts`: narrow typed IPC contracts, with CP and tag IDs represented as decimal strings.
- `src/ReferencePanel.tsx`: reference resolution, metadata/provenance and explicit saved species/form confirmation; independent of display labels and OCR.
- `src-tauri/src/reference.rs` and `src-tauri/reference/catalog.json`: validated, versioned offline reference snapshot and conservative identity resolution. The production manifest is empty pending verified redistribution rights; test fixtures are fictional.
- `src-tauri/src/screenshot.rs`: native picker, bounded local image validation, extraction/crop preparation before downsampling and metadata-free pixel preview.
- `src-tauri/src/extraction.rs`: pure bounded pixel analysis, typed field suggestions and recoverable unresolved/unavailable notices; no repository or filesystem access.
- `src-tauri/src/ocr_regions.rs`: bounded temporary OCR crops anchored to supported detail-screen layout and current-HP evidence.
- `src/ocr.ts` / `src/ocr-policy.ts`: isolated bundled Tesseract worker, cancellation/timeout, local-only resources and typed text suggestions; no persistence.
- `src-tauri/src/commands.rs`: typed inputs, safe error mapping and presentation DTOs; no image fields in save inputs.
- `src-tauri/src/database.rs`: initialization and immutable migrations.
- `src-tauri/src/inventory.rs`: domain validation, structured persistence, atomic record/tag creation and IV-only correction with a transactional comparison of expected saved values.
- `src-tauri/src/evaluation.rs`: deterministic, versioned appraisal-only rules, typed findings and a narrow future recommendation contract. No strategic data or network dependency.
- `src-tauri/build.rs`, `permissions/` and `capabilities/default.json`: command ACL restricted to the local main window. No generic filesystem, shell, process, HTTP, SQL or dialog plugin access.

The renderer never executes SQL or supplies a file/database path. Images stay in temporary native/renderer memory; only explicitly confirmed structured data is persisted. See [screenshot review](../development/screenshot-review.md) for limits and contracts. Name/CP OCR runs in a same-origin worker using bundled engine/language resources; broader layout recognition remains deferred. See [local extraction](../development/local-extraction.md) for the deliberately narrow supported evidence and confidence semantics.

Migration 4 adds IV revision tracking and per-record appraisal attempt metadata. Rust derives lifecycle state and recalculates current descriptive results; totals, percentages, recommendations and duplicate input snapshots are not stored. The narrow `evaluate_pokemon` command accepts a validated UUID, evaluates confirmed data and persists the attempt atomically under serialized native access. The existing main-window ACL grants this command without generic storage or network access. See [data model](data-model.md), [evaluation workflow](../development/evaluation.md) and [decision 0005](../decisions/0005-persisted-appraisal-evaluation.md).

The follow-up `correct_pokemon_ivs` command accepts only a UUID and explicit expected/corrected IV groups. An immediate transaction validates and compares the saved group, updates only changed IVs and the update timestamp, and returns the record with its tags and derived evaluation state. The existing migration-4 trigger handles invalidation. No schema or engine-version change is required, and correction does not run evaluation.

Task 006 adds migration 5 for separately confirmed reference identity and stable dataset/species/form pins. `correct_pokemon_identity` compares expected identity before writing that association only. Catalog updates preserve player observations; reference metadata is derived at read time. Pure `evaluation::assess` accepts reference context alongside IVs, while the existing appraisal rules and lifecycle remain unchanged. See [reference data](../development/reference-data.md) and [decision 0006](../decisions/0006-versioned-local-reference-data.md).

## Confirmed individual lifecycle (S2.2)

`src/LifecyclePanel.tsx` adds selected-UUID evolution, targeted screenshot review and confirmed-history display. `src-tauri/src/inventory/lifecycle.rs` owns transactional catalog validation, expected-observation checks, patch validation and confirmed audit insertion. New commands are explicitly listed in the existing main-window command permission; no generic storage or filesystem access is granted. Migration 7 adds one event table, reusing the established repository and strategic observation triggers. See [lifecycle contracts](../development/lifecycle.md).

S2.3 adds `src/PowerUpPanel.tsx` to the same detail/action lock and `confirm_power_up` to the existing native command boundary. Preview and cancellation remain transient. Confirmation reuses request UUID replay protection, observation revisions and atomic history, updating only explicitly supplied CP. Migration 8 preserves existing events while extending the table's checked action kinds. No resource management, screenshot pipeline or game mechanics calculation is added.

## Proposed strategic boundary (Task 008)

This section is a design proposal, not an implemented module or IPC API. Task 007's offline adapter, native validation, SHA-256 provenance and cumulative identity/version ledger are merged. Production still loads `tcm-empty` / `empty-1`; no strategic calculation is available. Schema 1 contains nullable types/base stats and descriptive evolution requirements, with no executable eligibility rules, move mechanics, level multipliers or investment costs. Do not parse those strings into eligibility or assume missing data means zero cost.

Proposed flow: confirmed individual observations + selected goal/budget/protections + reviewed versioned mechanics → native deterministic validation and feasible candidate generation → optional project-trained suitability scoring → deterministic constraint recheck and path sequencing → typed assessment/roadmap presentation → explicit save/progress confirmation. Missing required facts block an investment path. The deterministic result remains usable without a model; invalid or uncertain model output falls back or abstains.

- Native domain logic owns role/readiness explanations, targets, prerequisites, conditional alternatives and resource totals. A separate strategic contract replaces the future-use single `proposedClassification` vocabulary for this scope; do not overload the implemented appraisal DTO or change its empty recommendation behavior.
- Native persistence owns observation/goal revisions, minimal plan snapshots, current/prior plan rotation, action reports and expected-revision conflict checks. Reference updates cannot overwrite player observations. No model writes inventory or SQL.
- React presents current and potential uses, the selected path, use-now alternatives, missing data, estimates/unknowns, provenance and separate action-report/observation-confirmation controls. Completion reports cannot fabricate observations or trigger game actions. Assigned tags remain user-owned.
- A future versioned model artifact declares feature/label schema, training provenance, compatible mechanics/datasets, digest and license. It consumes validated structured features and returns bounded suitability scores/abstention, never reference facts or executable tools. Offline resource limits and fallback are release gates.

Keep reference facts, mechanics, recommendation labels and player data as separate provenance domains. Any future mechanics schema must retain Task 007 identity continuity and release review, without silently changing schema 1 or the production snapshot. See [decision 0007](../decisions/0007-individual-progression-roadmaps.md), [proposed storage](data-model.md#proposed-strategic-assessments-and-roadmaps-task-008) and [evaluation gates](../development/evaluation.md#proposed-strategic-and-learning-evaluation-task-008).
