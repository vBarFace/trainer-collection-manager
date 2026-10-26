# Product requirements document

## Vision

Trainer Collection Manager is a free, open-source, offline-first desktop app for Pokémon GO players to maintain a local collection and receive explainable support for manual in-game decisions.

## Foundation scope

This release provides a Windows Tauri shell, private local inventory and manual screenshot review. Users select one local PNG/JPEG, inspect a temporary preview and experimental local IV suggestions, enter/correct supported data fields, then explicitly confirm saving. Review confirms data only; classification tags belong to a later system evaluation. Inventory lists persisted records under **Awaiting evaluation**, including after restart, and preserves any existing tags. All current records are unevaluated; evaluation and automatic tagging are not implemented. Name/CP OCR, broad screenshot recognition, account integration, recommendations, teams, simulations, local LLM runtime, datasets, telemetry and cloud capabilities are deferred.

## Product principles

- Local-first and useful without AI or a network connection.
- User is always in control: recommendations are explainable and actions occur manually in Pokémon GO.
- Persist structured, reviewed data only. Never permanently retain screenshots or delete originals without explicit consent.
- Prefer deterministic, versioned formulas and simulation where appropriate.

## Intended import and evaluation flow

Select a screenshot → attempt local appraisal-bar extraction → review/correct the data → confirm saving → await evaluation → evaluate and propose classification tags/recommendations. Name and CP require manual entry; unresolved IVs remain empty. Experimental geometry matches are tentative suggestions, with no measured real screenshot accuracy. Review confirmation does not assert that the Pokémon has been evaluated. Existing user-assigned tags do not establish evaluation status.

## Future constraints

Each Pokémon instance must have a stable UUID. Standard tags are `EVOLVE`, `POWER UP`, `ATTACKER`, `GYM DEF`, `ROCKET`, `PVP 1500`, `PVP 2500`, `TRANSFER`, and `SAVE/SPECIAL`; custom tags are allowed. IV appraisal and second attacks are fields/features, not default tags.
