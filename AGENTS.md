# Repository guidance

## Architecture

- Keep one local-first Tauri 2 / React / TypeScript / Rust repository. The native layer owns persistence; the UI never executes arbitrary SQL.
- Store user data only in the Tauri per-user app-data directory. Do not add cloud services, telemetry, game automation, accounts, or broad filesystem permissions without an approved decision record.
- Add SQLite changes as ordered, immutable migrations in `src-tauri/src/database.rs`, with tests. Every future individual Pokémon needs a stable UUID; document schema invariants before implementation.
- Prefer deterministic and explainable domain logic. Any future AI action needs typed tools, preview and explicit user confirmation.

## Conventions and quality

- Use English for code, comments and documentation; strict TypeScript; semantic accessible UI.
- Use npm and commit `package-lock.json`. Do not add dependencies without a clear need and license review.
- Run `npm run check`, `npm run rust:fmt`, and `npm run rust:check` before requesting review. Run `npm run tauri:build` when native prerequisites are available.
- Do not commit database files, screenshots, personal data, secrets, model weights, build output, or unreviewed third-party assets.

## Git workflow

- Never commit directly to `main`. Use a focused branch and reviewable conventional commits.
- Do not force-push, rewrite history, change repository settings, or push/open/merge a PR without explicit approval.
- Keep PRs scoped; update documentation when behavior, architecture or privacy boundaries change.

## Licensing and privacy

- Original code is MIT. Pokémon, Pokémon GO, Niantic, images, sprites, datasets and model weights are not covered by that license; review each source before inclusion.
- Do not retain user screenshots by default and never delete originals without explicit authorization.
