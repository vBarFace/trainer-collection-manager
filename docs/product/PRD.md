# Product requirements document

## Vision

Trainer Collection Manager is a free, open-source, offline-first desktop app for Pokémon GO players to maintain a local collection and receive explainable support for manual in-game decisions.

## Foundation scope

This release establishes a Windows Tauri shell and private local metadata database. Navigation is placeholder-only. No Pokémon records, account integration, screenshot ingestion, OCR, recommendations, teams, simulations, local LLM runtime, datasets, telemetry or cloud capability is implemented.

## Product principles

- Local-first and useful without AI or a network connection.
- User is always in control: recommendations are explainable and actions occur manually in Pokémon GO.
- Persist structured, reviewed data only. Never permanently retain screenshots or delete originals without explicit consent.
- Prefer deterministic, versioned formulas and simulation where appropriate.

## Future constraints

Each Pokémon instance must have a stable UUID. Initial standard tags are `EVOLUIR`, `POWER UP`, `ATACANTE`, `DEF GYM`, `ROCKET`, `PVP 1500`, `PVP 2500`, and `TRANSFERIR`; custom tags are allowed. IV appraisal and second attacks are fields/features, not default tags.
