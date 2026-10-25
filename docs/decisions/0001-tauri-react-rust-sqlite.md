# ADR 0001: Tauri 2, React, Rust and SQLite

**Status:** Accepted

Use Tauri 2 with a React/TypeScript renderer and Rust native layer. Use SQLite local to the application's per-user data directory for durable data.

This meets Windows-first desktop needs without a server, supports small distributable builds, and establishes a safe native persistence boundary. Full data-model design is deliberately deferred.
