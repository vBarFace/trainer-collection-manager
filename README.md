# Trainer Collection Manager

An open-source, offline-first Windows desktop foundation for Pokémon GO collection management. This project is independent and is not affiliated with or endorsed by Pokémon or Niantic.

## What works today

- Tauri 2 + React + TypeScript desktop shell with keyboard-accessible navigation.
- Local SQLite initialization in the per-user application-data directory.
- Ordered SQLite migrations for application settings and normalized Pokémon inventory/tags.
- Native typed Rust inventory CRUD and tagging APIs, including optional appraisal IVs and the `SAVE/SPECIAL` standard tag, with validation and persistence tests.
- Strict frontend linting, formatting, typechecking, unit-test and CI configuration.

The Inventory, Recommendations, Teams, Bag and Settings views are intentionally placeholders. Inventory persistence exists in Rust only; no inventory UI or renderer commands are connected yet. There is no screenshot/OCR processing, AI, telemetry, cloud service, game automation, data import or external dataset in this release.

## Quick start (Windows)

Install Node.js LTS, Rust with the stable MSVC toolchain, Visual Studio Build Tools with **Desktop development with C++**, and Microsoft Edge WebView2. Then:

```powershell
npm ci
npm run tauri:dev
```

See [development setup](docs/development/setup.md) for all commands and prerequisites.

## Privacy and data

The application is designed to be useful without an account or network connection. Its SQLite database is created under the application data directory resolved by Tauri; it is excluded from Git. Screenshots and user originals are not handled by this foundation.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and the repository [AGENTS.md](AGENTS.md). Original project code is available under the [MIT License](LICENSE); third-party materials need separate review.
