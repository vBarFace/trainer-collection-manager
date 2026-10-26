# Architecture overview

The desktop process uses Tauri 2 and Rust. React + TypeScript renders accessible local inventory review and placeholder navigation. Rust resolves the per-user application-data directory, opens `trainer-collection-manager.sqlite3`, enables foreign keys/WAL and applies ordered migrations.

- `src/Inventory.tsx`: manual data review without tag selection, explicit save confirmation and persisted records presented as awaiting evaluation. Existing tags are displayed without implying completed evaluation.
- `src/native.ts`: narrow typed IPC contracts, with CP and tag IDs represented as decimal strings.
- `src-tauri/src/screenshot.rs`: native picker, bounded local image validation and metadata-free pixel preview.
- `src-tauri/src/commands.rs`: typed inputs, safe error mapping and presentation DTOs; no image fields in save inputs.
- `src-tauri/src/database.rs`: initialization and immutable migrations.
- `src-tauri/src/inventory.rs`: domain validation, structured persistence and atomic record/tag creation.
- `src-tauri/build.rs`, `permissions/` and `capabilities/default.json`: command ACL restricted to the local main window. No generic filesystem, shell, process, HTTP, SQL or dialog plugin access.

The renderer never executes SQL or supplies a file/database path. Images stay in temporary native/renderer memory; only explicitly confirmed structured data is persisted. See [manual screenshot review](../development/screenshot-review.md) for limits and contracts. OCR and automatic extraction remain deferred.
