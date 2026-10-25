# Product requirements document

## Vision

Trainer Collection Manager is a free, open-source, offline-first desktop app for Pokémon GO players to maintain a local collection and receive explainable support for manual in-game decisions.

## Foundation scope

This release provides a Windows Tauri shell, private local inventory and manual screenshot review. Users select one local PNG/JPEG, review a temporary preview, manually enter supported fields/tags and explicitly confirm saving. Inventory lists persisted records after restart. OCR and automatic extraction, account integration, recommendations, teams, simulations, local LLM runtime, datasets, telemetry and cloud capabilities are deferred.

## Product principles

- Local-first and useful without AI or a network connection.
- User is always in control: recommendations are explainable and actions occur manually in Pokémon GO.
- Persist structured, reviewed data only. Never permanently retain screenshots or delete originals without explicit consent.
- Prefer deterministic, versioned formulas and simulation where appropriate.

## Future constraints

Each Pokémon instance must have a stable UUID. Standard tags are `EVOLVE`, `POWER UP`, `ATTACKER`, `GYM DEF`, `ROCKET`, `PVP 1500`, `PVP 2500`, `TRANSFER`, and `SAVE/SPECIAL`; custom tags are allowed. IV appraisal and second attacks are fields/features, not default tags.
