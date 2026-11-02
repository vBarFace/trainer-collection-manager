# Individual evolution and Power Up planner

Implemented in the `collection-4` / `preparation-1` increment. The individual
UUID remains the anchor for recommendations, the saved roadmap and confirmed
game actions. No new OCR fields, cloud services, dependencies or game records
are bundled.

## User flow

1. Import local PoGoAPI reference files in Settings. The previous seven-file gym
   input remains accepted. Add `pokemon_evolutions.json`,
   `pokemon_powerup_requirements.json`, `pvp_fast_moves.json` and
   `pvp_charged_moves.json` for preparation and Trainer-Battle signals. Every
   selected file retains its source URL, revision/date declaration and SHA-256.
   Use a new revision label when extending a previously imported input set; the
   immutable version ledger rejects different contents under an existing version.
2. Evaluate an individual. Review its independent gym attack, defense, Rocket,
   PvP and transfer-retention assessments. Unknown role value never certifies
   safe transfer. Defense uses natural bulk and resistances, Rocket uses actual
   Trainer-Battle energy/turn mechanics, and PvP keeps stat-product and move-kit
   signals distinct from competitive matchup strength.
3. Optionally confirm current level/moves and planning allocations. Candy and XL
   refer to this evolution family's resources; Stardust is shared globally.
   Allocations are an amount the user chooses to consider for this individual,
   not separate wallets or reservations. Blank is unknown; zero is known zero.
   Confirm the ordinary cost profile and in-game Power Up cap when applicable.
4. Compare retaining the current state with complete evolution/move/Power Up
   alternatives. The selected goal controls the move model and target context.
   Levels 30/40/50 are comparison checkpoints, not optimal investment levels.
   Great/Ultra targets respect the projected CP cap under the imported CPM grid.
5. Select a path for the roadmap and recalculate. Actions appear in order.
   Review opens the existing individual workflow. Confirm an evolution or
   Power Up only after actually doing it in game; observe changed attacks
   manually. No progress button fabricates CP, level, species, attacks or spend.
6. Facts, source activation, allocations, goal changes and actual actions stale
   the saved plan. Recalculation preserves the previous roadmap as history and
   resets progress. Route identity includes the role, evolution chain, levels,
   actions, costs and source; a changed goal cannot silently select another route.

## Cost and benefit boundaries

Power Up totals sum explicit half-level source rows. Candy, Candy XL and Stardust
are separate amounts. Missing rows/start level/profile yield unknown totals,
not zeros. Evolution Candy is added once for each branch. Item and additional
prerequisites are shown alongside the path. Trade discounts are conditional;
no trade history, quests, gender, location, time or event completion is inferred.
The evolution source is not complete enough to certify all in-game prerequisites.

Known unsupported variants do not receive ordinary numeric battle projections.
Conflicting observed CP and level/stat evidence require reconfirmation. Missing
target CPM leaves a cost-only conditional alternative; it never establishes
an evaluated Power Up benefit. In particular, the current PoGoAPI adapter still
withholds documented approximate CPM above40 while accepting explicit cost rows
through50, allowing XL costs without invented high-level CP/damage.

Gym preparation compares supported current confirmed attacks (including the
second charged slot) with the proposed set against fixed Defense215 at level30.
The source CPM ratio changes with the planned attacker level. This is one
neutral zero-energy cycle, not actual DPS/TDO or a win-rate. Future-form role
signals also preserve the distinction between potential and current readiness.

Ordinary move-pool membership indicates a normal TM route, with an unknown number
of random attempts. Elite lists identify conditional Elite TM routes. Neither
list proves a current evolution/event window. Unknown, special, Frustration and
unsupported options require in-game verification. Proposed attacks are never
written to individual observations.

The display is bounded to16 alternatives and up to3 future forms. When branches
exceed that bound, a goal-specific proxy selects the displayed subset and the
limitation is explicit. This is a decision aid, not exhaustive path optimization.
Opponent simulation, competitive meta rankings, exact event/species cost overrides,
automatic transfers and automatic resource accounting remain outside this model.

## Persistence and compatibility

Existing migrations are unchanged. Optional allocations use existing native
`application_settings` entries keyed by the canonical individual UUID. Values
are typed, bounded and compare-and-swap revisioned; deletion cleans the key in
the same transaction as the individual. Selection validates the current snapshot
fingerprint in an immediate transaction. Allocations and selection join the
roadmap context hash and the structured user-data export. No screenshot metadata
or downloaded source documents are included in that export.

New reference/report properties default to absent for older packages and saved
roadmaps. Old documents remain readable and the ruleset change makes old plans
stale. Native SQLite snapshots couple identity pins, details, constraints and
the active reference. No UI SQL or resource mutation is exposed.

## Primary references

- [PoGoAPI endpoint schemas](https://pogoapi.net/documentation/)
- [Niantic gym battle context](https://niantic.helpshift.com/hc/en/6-pokemon-go/faq/83-battling-at-gyms/)
- [Niantic Rocket battle context](https://niantic.helpshift.com/hc/en/6-pokemon-go/faq/1936-team-go-rocket-grunts/)

Public accessibility is not a grant to redistribute Pokémon GO data. Tests use
original fictional specimens and mechanics; real-source validation stays private.

## Executed validation — 2026-11-02

- `npm run check`: formatting, lint, TypeScript, 158 UI tests and8 adapter tests
  passed. The final compact alternative presentation is included in this run.
- Native all-target/reference-tools suite:200 passed,2 authorized private-image
  tests ignored. Cases include exact half-level sums, XL boundary, evolution
  plus Power Up, missing/zero resources, both charged slots, Shadow abstention,
  missing target CPM, role-bound route identity, selected-path persistence,
  stale observations, same-UUID actions and the known species cost override.
- `npm run reference:test-native`:2 passed. Rust format/check passed.
- All11 privately acquired source files passed native normalization:965 ordinary
  forms,448 supported evolution branches and98 explicit cost intervals. Three
  hypothetical specimens exercised collection analysis and bounded saved-roadmap
  persistence. All23 ordinary-specimen path costs matched an independent raw-row
  calculation; the special-cost specimen correctly withheld its totals.
- `npm run tauri:build`: production frontend, Rust release and Windows x64 NSIS
  installer built successfully. Installer bytes:12,419,058; SHA-256
  `306e7cf8fc79a4ac98cfd133c4d2a9c6fa89d0afb74e23c672595d86e1f8779e`.

The installer wizard and a real-collection installed-app journey were not run.
These checks verify software behavior and model arithmetic, not actual battle
outcomes, a competitive meta model or complete evolution prerequisite coverage.

## Consistency follow-up — collection-5 / roles-3

The user requires stable collection-wide reassessment. A regression reproduced
an order-dependent tie in Trainer-move suggestions; source IDs now break equal
modeled metrics consistently. The aggregate ruleset version makes this deliberate
change visible and invalidates earlier saved plans once.

Native coverage now includes identical serialized collection results across
repeated unchanged assessment, unchanged-value saves and reopening the native
repository. A real budget change produces a changed result that is then stable
on repeated assessment. Adding an unrelated weaker specimen changes the snapshot
fingerprint without changing the existing individual's advice. These checks also
confirm that assessment does not rewrite observed Pokémon facts or applied tags.

Executed:203 native tests passed with2 private-image tests ignored;
`npm run check` passed158 UI and8 adapter tests; Rust format/check and the Windows
NSIS release build passed. The new installer is12,421,655 bytes, SHA-256
`3826934a022222edef23b3b01f593655144ac3a4147ef799129d99c413b5f6bd`.

The collection-wide change-review UI and its material-change policy remain future
work under the [consistency contract](../product/individual-decision-loop.md#consistency-is-required-for-collection-wide-review).
