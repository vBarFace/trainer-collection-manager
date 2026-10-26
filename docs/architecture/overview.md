# Architecture overview

The desktop process uses Tauri 2 and Rust. React + TypeScript renders accessible local inventory review and placeholder navigation. Rust resolves the per-user application-data directory, opens `trainer-collection-manager.sqlite3`, enables foreign keys/WAL and applies ordered migrations.

- `src/Inventory.tsx`: editable data review with tentative extraction provenance and no tag selection, explicit save confirmation, appraisal lifecycle display and record selection for evaluation/re-evaluation. Existing assigned tags remain separate from findings and recommendations.
- `src/AppraisalPanel.tsx`: accessible appraisal details, current inputs/results, ruleset provenance, missing data and limitations; presentation only, with no duplicated evaluation rules.
- `src/native.ts`: narrow typed IPC contracts, with CP and tag IDs represented as decimal strings.
- `src-tauri/src/screenshot.rs`: native picker, bounded local image validation, extraction/crop preparation before downsampling and metadata-free pixel preview.
- `src-tauri/src/extraction.rs`: pure bounded pixel analysis, typed field suggestions and recoverable unresolved/unavailable notices; no repository or filesystem access.
- `src-tauri/src/ocr_regions.rs`: bounded temporary OCR crops anchored to supported detail-screen layout and current-HP evidence.
- `src/ocr.ts` / `src/ocr-policy.ts`: isolated bundled Tesseract worker, cancellation/timeout, local-only resources and typed text suggestions; no persistence.
- `src-tauri/src/commands.rs`: typed inputs, safe error mapping and presentation DTOs; no image fields in save inputs.
- `src-tauri/src/database.rs`: initialization and immutable migrations.
- `src-tauri/src/inventory.rs`: domain validation, structured persistence and atomic record/tag creation.
- `src-tauri/src/evaluation.rs`: deterministic, versioned appraisal-only rules, typed findings and a narrow future recommendation contract. No strategic data or network dependency.
- `src-tauri/build.rs`, `permissions/` and `capabilities/default.json`: command ACL restricted to the local main window. No generic filesystem, shell, process, HTTP, SQL or dialog plugin access.

The renderer never executes SQL or supplies a file/database path. Images stay in temporary native/renderer memory; only explicitly confirmed structured data is persisted. See [screenshot review](../development/screenshot-review.md) for limits and contracts. Name/CP OCR runs in a same-origin worker using bundled engine/language resources; broader layout recognition remains deferred. See [local extraction](../development/local-extraction.md) for the deliberately narrow supported evidence and confidence semantics.

Migration 4 adds IV revision tracking and per-record appraisal attempt metadata. Rust derives lifecycle state and recalculates current descriptive results; totals, percentages, recommendations and duplicate input snapshots are not stored. The narrow `evaluate_pokemon` command accepts a validated UUID, evaluates confirmed data and persists the attempt atomically under serialized native access. The existing main-window ACL grants this command without generic storage or network access. See [data model](data-model.md), [evaluation workflow](../development/evaluation.md) and [decision 0005](../decisions/0005-persisted-appraisal-evaluation.md).
