# Trainer Collection Manager

An open-source, offline-first Windows desktop foundation for Pokémon GO collection management. This project is independent and is not affiliated with or endorsed by Pokémon or Niantic.

## What works today

- Tauri 2 + React + TypeScript desktop shell with keyboard-accessible navigation.
- Local SQLite initialization in the per-user application-data directory.
- Ordered SQLite migrations for application settings and normalized Pokémon inventory/tags.
- Native typed Rust inventory CRUD and tagging APIs, including optional appraisal IVs and the `SAVE/SPECIAL` standard tag, with validation and persistence tests.
- Native PNG/JPEG selection, local preview, automatic local name/CP OCR and appraisal-bar suggestions and editable review with explicit confirmed inventory saving and a persistent list.
- Versioned offline appraisal evaluation, persistent lifecycle states and explainable IV results. Select an inventory record to evaluate or re-evaluate it.
- Explicit correction of saved appraisal IVs, including marking the group unknown, with cancellation, retry and conflicting-edit protection.
- Validated/versioned offline reference foundation and explicit species/form confirmation, separate from display names. This release uses an empty catalog pending verified dataset rights; real GO metadata is unavailable. See [reference data](docs/development/reference-data.md).
- Reproducible offline PvPoke development adapter, native candidate validation, SHA-256 provenance and identity continuity checks. Original fictional fixtures exercise generation; the real-data release criterion remains blocked by unverified dataset redistribution rights. See [adapter workflow](docs/development/reference-data.md#task-007-adapter-and-release-workflow).
- Strict frontend linting, formatting, typechecking, unit-test and CI configuration.

Inventory supports [screenshot review](docs/development/screenshot-review.md) and confirmed saving of display name, optional CP and optional grouped appraisal IVs. Imported records start **Pending**. [Appraisal evaluation](docs/development/evaluation.md) reports attack/defense/HP IVs, their total and descriptive percentage, and a perfect 15/15/15 finding when applicable. Missing IVs produce **Needs data**; IV changes or a ruleset change make previous attempts **Stale**. Select **Correct saved IVs** to review and confirm a replacement group, then request evaluation separately. Names, CP and tags are outside this correction flow. Evaluation metadata and confirmed corrections survive restart.

Appraisal percentage is not a PvP ranking, PvE strength score, overall quality score or transfer criterion. No strategic recommendation or suggested tag is generated, and evaluation never changes assigned tags. Import review still confirms data only. Recommendations, Teams, Bag and Settings remain placeholders. Name, visible CP and appraisal IVs are filled automatically when supported evidence is readable. OCR and bar suggestions remain editable and tentative, with manual fallback. One supplied real screenshot has been verified; general recognition accuracy has not been established. See [extraction limitations](docs/development/local-extraction.md). Conversational AI, cloud, telemetry, game automation and external datasets are deferred.

## Quick start (Windows)

Install Node.js LTS, Rust with the stable MSVC toolchain, Visual Studio Build Tools with **Desktop development with C++**, and Microsoft Edge WebView2. Then:

```powershell
npm ci
npm run tauri:dev
```

See [development setup](docs/development/setup.md) for all commands and prerequisites.

## Privacy and data

The application is designed to be useful without an account or network connection. Its SQLite database is created under the application data directory resolved by Tauri; it is excluded from Git. Screenshots are opened read-only, validated and processed in memory. The OCR engine and English/Portuguese language resources are included at build time; runtime requires no download or internet connection. Original image bytes, metadata, paths and filenames are never stored in the application database or copied into application storage. Only explicitly confirmed structured records and tags are saved.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and the repository [AGENTS.md](AGENTS.md). Original project code is available under the [MIT License](LICENSE); third-party materials need separate review.
