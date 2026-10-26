# Product requirements document

## Vision

Trainer Collection Manager is a free, open-source, offline-first desktop app for Pokémon GO players to maintain a local collection and receive explainable support for manual in-game decisions.

## Foundation scope

This release provides a Windows Tauri shell, private local inventory, screenshot review, confirmed saved-IV corrections and deterministic appraisal evaluation. Users select one local PNG/JPEG, inspect a temporary preview and local name/CP/IV suggestions, enter/correct supported data fields, then explicitly confirm saving. Review confirms data only and creates a pending record. Users can select a saved Pokémon, correct its appraisal IV group and request an appraisal evaluation or re-evaluation separately. Inventory preserves existing records, UUIDs and assigned tags, and presents Pending, Evaluated, Needs data or Stale appraisal states after restart. Broad screenshot recognition, strategic recommendations, automatic tagging, account integration, teams, simulations, local LLM runtime, datasets, telemetry and cloud capabilities are deferred.

## Product principles

- Local-first and useful without AI or a network connection.
- User is always in control: recommendations are explainable and actions occur manually in Pokémon GO.
- Persist structured, reviewed data only. Never permanently retain screenshots or delete originals without explicit consent.
- Prefer deterministic, versioned formulas and simulation where appropriate.

## Intended import and evaluation flow

Select a screenshot → attempt local name/CP/IV extraction → review/correct the data → confirm saving → select the pending record → request appraisal evaluation → review facts, findings and limitations. Name, visible CP and IVs are suggested automatically for supported layouts; unreadable or hidden fields remain unresolved and can be entered manually. Experimental geometry matches are tentative suggestions, validated on one supplied real example, with no general accuracy measurement. Review confirmation does not assert that the Pokémon has been evaluated. Existing user-assigned tags do not establish evaluation status.

## Appraisal evaluation

- The offline Rust engine uses confirmed appraisal IVs only. All three values are required; unknown values remain unknown and produce Needs data with explicit missing fields.
- Known values yield attack, defense and HP IVs (0–15), total appraisal IVs (0–45), and appraisal percentage (`total × 100 / 45`). A 15/15/15 appraisal produces a noteworthy perfect-appraisal finding.
- These facts describe appraisal only. They do not establish PvP rank, PvE strength, overall quality, transfer suitability or universal competitive superiority. PvP, PvE, gym, Rocket and investment analysis have not been performed.
- Each evaluation attempt has a ruleset version and UTC timestamp. Changes to actual IV values or a ruleset mismatch make it Stale; re-evaluation replaces the attempt metadata. Name, CP and tag changes do not affect this scope. Stale results are withheld until re-evaluation.
- Findings, future recommendations/suggested tags and assigned tags are distinct. This engine emits no recommendations and never changes tags or infers shiny status or user preferences.
- Selection, IV correction and evaluation controls support keyboard use, loading, safe errors and retry. Full inventory editing and filtering remain deferred.

## Saved appraisal-IV correction

- Select a saved record and choose **Correct saved IVs**. The form starts with the confirmed values. Supply all three integers from 0 to 15, or clear all three to mark the group unknown, then explicitly confirm. Partial groups are rejected; cancelling or switching records discards the unsaved draft.
- Only IVs can be changed. Names, CP, UUIDs, creation timestamps and tags are preserved. Changed IVs invalidate a prior appraisal through the existing lifecycle; unchanged values preserve its state. Saving never runs evaluation automatically.
- Save failures retain the draft for retry. If current saved IVs differ from those displayed when correction began, reject the save and require a reload before retrying that draft. Missing/deleted records also require reload. No screenshot selection or extraction is needed for correction.

## Future constraints

Each Pokémon instance must have a stable UUID. Standard tags are `EVOLVE`, `POWER UP`, `ATTACKER`, `GYM DEF`, `ROCKET`, `PVP 1500`, `PVP 2500`, `TRANSFER`, and `SAVE/SPECIAL`; custom tags are allowed. IV appraisal and second attacks are fields/features, not default tags.

Future strategic recommendations require validated species/form identification, explicit structured moves/level and other applicable observations, licensed and versioned game data, validated formulas, and user goals where relevant. They must have their own scope and provenance. CP or aggregate IV percentage alone must never justify transfer, evolution or power-up.
