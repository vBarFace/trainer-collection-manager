# Architecture overview

## Implemented

The desktop process uses Tauri 2 and Rust. React + TypeScript is the renderer, built by Vite. The Rust startup path resolves Tauri's per-user application-data directory, opens `trainer-collection-manager.sqlite3`, enables foreign keys/WAL, and applies ordered migrations. The renderer has no direct database or filesystem access.

## Boundaries

- `src/`: accessible presentation only.
- `src-tauri/src/database.rs`: typed persistence initialization and migrations.
- `src-tauri/src/inventory.rs`: native typed inventory repository and validation; no renderer commands yet.
- `src-tauri/capabilities/default.json`: least-privilege capability allowlist; no filesystem, shell, process, HTTP or SQL-plugin access.

Future UI/native communication must use narrow, typed commands. Never expose arbitrary SQL, file paths or unrestricted LLM tools to the renderer.
