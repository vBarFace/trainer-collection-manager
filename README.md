# Trainer Collection Manager

An open-source, offline-first Windows desktop foundation for Pokémon GO collection management. This project is independent and is not affiliated with or endorsed by Pokémon or Niantic.

## What works today

- Tauri 2 + React + TypeScript desktop shell with keyboard-accessible navigation.
- Local SQLite initialization in the per-user application-data directory.
- Ordered SQLite migrations for application settings and normalized Pokémon inventory/tags.
- Native typed Rust inventory CRUD and tagging APIs, including optional appraisal IVs and the `SAVE/SPECIAL` standard tag, with validation and persistence tests.
- Native PNG/JPEG selection, local preview, experimental local appraisal-bar suggestions and editable review with explicit confirmed inventory saving and a persistent list.
- Strict frontend linting, formatting, typechecking, unit-test and CI configuration.

Inventory supports [screenshot review](docs/development/screenshot-review.md) and confirmed saving of display name, optional CP and optional grouped appraisal IVs. Confirmed records appear in **Awaiting evaluation**. Import review confirms data only; it does not ask for classification tags. Previously saved tags remain visible. Evaluation and automatic tagging are deferred. Recommendations, Teams, Bag and Settings remain placeholders. Name/CP OCR remains unavailable. Experimental IV extraction supports only a narrow bar geometry and palette, with tentative confidence and manual fallback; real screenshot accuracy has not been established. See [extraction limitations](docs/development/local-extraction.md). AI, cloud, telemetry, game automation and external datasets are deferred.

## Quick start (Windows)

Install Node.js LTS, Rust with the stable MSVC toolchain, Visual Studio Build Tools with **Desktop development with C++**, and Microsoft Edge WebView2. Then:

```powershell
npm ci
npm run tauri:dev
```

See [development setup](docs/development/setup.md) for all commands and prerequisites.

## Privacy and data

The application is designed to be useful without an account or network connection. Its SQLite database is created under the application data directory resolved by Tauri; it is excluded from Git. Screenshots are opened read-only, validated and previewed in memory. Original image bytes, metadata, paths and filenames are never stored in the application database or copied into application storage. Only explicitly confirmed structured records and tags are saved.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and the repository [AGENTS.md](AGENTS.md). Original project code is available under the [MIT License](LICENSE); third-party materials need separate review.
