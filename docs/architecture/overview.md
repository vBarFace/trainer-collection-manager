# Architecture overview

The desktop process uses Tauri 2 and Rust. React + TypeScript renders accessible local inventory review and placeholder navigation. Rust resolves the per-user application-data directory, opens `trainer-collection-manager.sqlite3`, enables foreign keys/WAL and applies ordered migrations.

- `src/Inventory.tsx`: editable data review with tentative extraction provenance and no tag selection, explicit save confirmation and persisted records presented as awaiting evaluation. Existing tags are displayed without implying completed evaluation.
- `src/native.ts`: narrow typed IPC contracts, with CP and tag IDs represented as decimal strings.
- `src-tauri/src/screenshot.rs`: native picker, bounded local image validation, extraction/crop preparation before downsampling and metadata-free pixel preview.
- `src-tauri/src/extraction.rs`: pure bounded pixel analysis, typed field suggestions and recoverable unresolved/unavailable notices; no repository or filesystem access.
- `src-tauri/src/ocr_regions.rs`: bounded temporary OCR crops anchored to supported detail-screen layout and current-HP evidence.
- `src/ocr.ts` / `src/ocr-policy.ts`: isolated bundled Tesseract worker, cancellation/timeout, local-only resources and typed text suggestions; no persistence.
- `src-tauri/src/commands.rs`: typed inputs, safe error mapping and presentation DTOs; no image fields in save inputs.
- `src-tauri/src/database.rs`: initialization and immutable migrations.
- `src-tauri/src/inventory.rs`: domain validation, structured persistence and atomic record/tag creation.
- `src-tauri/build.rs`, `permissions/` and `capabilities/default.json`: command ACL restricted to the local main window. No generic filesystem, shell, process, HTTP, SQL or dialog plugin access.

The renderer never executes SQL or supplies a file/database path. Images stay in temporary native/renderer memory; only explicitly confirmed structured data is persisted. See [screenshot review](../development/screenshot-review.md) for limits and contracts. Name/CP OCR runs in a same-origin worker using bundled engine/language resources; broader layout recognition remains deferred. See [local extraction](../development/local-extraction.md) for the deliberately narrow supported evidence and confidence semantics.
