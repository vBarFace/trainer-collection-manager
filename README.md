# Trainer Collection Manager

An open-source, offline-first Windows desktop collection manager for Pokémon GO.
Original code is MIT. The project is independent of Pokémon and Niantic. No
account, telemetry, cloud service, game automation, LLM or learned recommendation
model is used.

## Current MVP preview

- Import up to 500 PNG/JPEG screenshots, extract queued images sequentially,
  correct uncertain name/CP/appraisal IV fields and explicitly group screenshots.
  Each confirmed new individual receives a stable UUID; retries are idempotent.
- Search and filter the inventory by name, confirmed species/form, CP, IV total,
  applied tags, appraisal status and confirmed properties. Inspect duplicates.
- Confirm optional HP, dimensions, gender, catch date, moves, level evidence and
  variants manually. Unknown stays unknown. Favorite, protection and applied tags
  are separate from engine suggestions.
- Install user-acquired local PvPoke, PokeMiners, PoGoAPI or reviewed Task 007
  schema-1 reference files in Settings. Validate and activate atomically, retain
  provenance, and roll back across restart. No game datasets are bundled.
- Read explainable Rust collection guidance and Great/Ultra/Master theoretical
  stat products within an explicit level-40 or level-50 mechanics ceiling. Compare
  actual owned alternatives. IV rank is not a battle-performance ranking.
- Set optional global collection/league goals and a confirmed Stardust budget.
  Inspect a per-UUID roadmap, postpone/ignore steps and open explicit observation
  confirmation. Stale plans block progress until recalculation.
- Confirm evolution, Power Up and targeted screenshot re-observation on the same
  UUID using the existing lifecycle workflows. No progress checkbox changes game
  observations or resources. Evolution needs a reviewed reference relationship.
- Restart with confirmed data and plans intact. Export structured user data,
  identity pins, tag definitions and plans without screenshots or catalog files.
  Backup import is not implemented.

## Supported scope and remaining acceptance

This is a bounded collection/statistical preview, **not the fully accepted gameplay
MVP**. Broader form normalization, merged field-level provider coverage, moves,
evolution eligibility and costs, PvE DPS/TDO, gym/Rocket simulation, meta rankings,
rarity/event evidence and executable investment plans remain unsupported. Extra
optional observations are manual; representative authorized screenshots are still
needed to benchmark and extend OCR. Current pinned sources now pass structural
adapter validation, with nine independent hypothetical numeric checks; broader
gameplay mechanics and real-image acceptance remain open. See the
[real-source validation record](docs/development/mvp-real-reference-validation.md).

See the [audit and delivery plan](docs/development/mvp-delivery.md),
[actual dataset coverage, acquisition and rights](docs/development/mvp-reference-coverage.md),
[batch import and inventory](docs/development/mvp-import-and-inventory.md),
[deterministic calculations](docs/development/mvp-strategy.md),
[roadmap confirmation/progress](docs/development/mvp-roadmaps.md), and
[executed validation](docs/development/mvp-alpha-validation.md).

## Quick start (Windows)

Install Node.js LTS, stable Rust MSVC, Visual Studio Build Tools with **Desktop
development with C++**, and Microsoft Edge WebView2. Then run:

```powershell
npm ci
npm run tauri:dev
```

For an installer, run `npm run tauri:build`. Build output is ignored by Git; no
release is published automatically. See [setup](docs/development/setup.md).

## Privacy and licensing

The native layer owns SQLite under the Tauri per-user application-data directory.
Screenshots are read-only and temporary; their bytes, metadata, names and paths
never enter persistent inventory. Original files remain untouched. Bundled generic
English/Portuguese OCR resources work offline. Reference imports use a native
picker and make no application network requests. Offline operation continues
after local data installation.

Public data accessibility and a software license do not establish underlying
Pokémon GO game-data rights. Keep downloaded datasets and user exports private
and outside Git/release bundles; see [third-party notices](docs/legal/third-party-assets.md).

Read [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md) and
[AGENTS.md](AGENTS.md). Original code is under [MIT](LICENSE).
