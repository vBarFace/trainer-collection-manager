# First gym-attack assessment and roadmap

Implemented scope: `collection-2` / `gym-attack-1`, 2026-11-02. This is a bounded
reference-potential assessment, not full gym battle simulation, actual readiness,
gym defense, Rocket performance or a transfer recommendation.

## Use the first slice

1. Obtain PoGoAPI JSON files yourself outside the repository/application bundle.
   In Settings choose PoGoAPI and select the files together (or a local envelope).
   Use a new dated source identifier when installing changed source content.
   Existing package versions cannot be reused with different normalized content.
2. Required catalog files are `pokemon_stats.json` and `pokemon_types.json`.
   Gym potential additionally needs `fast_moves.json`, `charged_moves.json`,
   `current_pokemon_moves.json` and `type_effectiveness.json`. Optional
   `cp_multiplier.json` enables the existing bounded league statistics. Up to seven
   files retain separate artifact hashes. The app itself makes no requests.
3. Import/review screenshots as before and explicitly confirm species/form.
   Display names can be nicknames; unresolved identities do not receive a gym build.
4. Open Recommendations and the individual's assessment. Gym potential comes
   before optional league statistics. See Pokémon typing, proposed attack sets,
   favorable target-type groups and typing vulnerabilities. Numerical model
   details and a few external benchmark options are expandable.
   A short qualitative comparison describes the imported same-type reference
   group; it is not an overall combat-strength label. Small subsets are flagged.
5. Optionally select the global **Gym attack preparation** goal. The reference
   calculation works with a single owned individual; an owned alternative only
   changes preparation priority. Favorite/protection/tag settings are preserved.
6. Create/recalculate the same UUID's roadmap. It names the proposed preparation
   set, offers optional review of actual attacks and reference comparison, and
   retains a wait/no-investment path. Review opens the selected individual's
   confirmation workspace; a planning checkbox cannot assert an in-game action.

Source endpoints:
[stats](https://pogoapi.net/api/v1/pokemon_stats.json),
[types](https://pogoapi.net/api/v1/pokemon_types.json),
[fast moves](https://pogoapi.net/api/v1/fast_moves.json),
[charged moves](https://pogoapi.net/api/v1/charged_moves.json),
[current pools](https://pogoapi.net/api/v1/current_pokemon_moves.json),
[type chart](https://pogoapi.net/api/v1/type_effectiveness.json),
[optional CPM](https://pogoapi.net/api/v1/cp_multiplier.json).
Public access does not establish game-data redistribution rights.

## What the result means

Potential uses an equal hypothetical level for both sides and a synthetic
defender with Defense 200 + IV15, not a selected actual gym defender. Actual Attack
IV is used if known; otherwise IV15 is explicitly hypothetical. Damage applies
the supplied type chart and STAB1.2. One cycle from zero energy uses the minimum
whole number of fast attacks needed for one charged attack, with supplied timings.
The details expose its average modeled damage per second, not observed DPS/TDO,
a win probability or the individual's current power. See [ADR 0010](../decisions/0010-reference-based-gym-potential.md).

The model evaluates imported ordinary reference forms, not just the collection.
Percentiles compare the supported subset within an attack type and scenario;
unreleased/obtainable status is not established. Benchmark names are alternative
reference options, not a list of legal gym defenders. Favorable type groups use
single-type opponents; dual typing and incoming attacks can change matchups.
No universal quality score, automatic applied tag, disposal or spending follows.

Optional observed moves can identify a displayed reference set already present
on the individual. Nonmatching/unknown moves remain proposed. Confirmed actions
invalidate old optional evidence and roadmaps, including a Power Up with no CP
patch. Fresh native snapshots resolve references consistently across instances.

## Provider gaps and compatibility

Real inputs contain misclassified positive-energy moves in the charged endpoint,
non-spending records, exceptional high-power records and duplicate display names
for distinct move IDs. Recovery requires independent current-pool confirmation;
unresolved options are withheld and recorded per form. No alias or energy cost is
guessed. Four current Normal pools reference an absent move in the inspected
source; those options remain explicitly unknown while supported attacks can be
assessed. Duplicate IDs, malformed values and wrong-kind links still fail closed.

Old catalog-only packages and stored roadmaps remain readable. Schema-1 identity
pins and UUIDs are preserved; there is no database migration. To enable the gym
model, old three-file PoGoAPI installations need a new complete combat import,
not merely a restart. PvPoke and GM imports remain catalog/statistical alternatives
in this increment; their PvP rankings are never relabeled gym evidence.

## Executed real-source and numerical validation

Seven bounded public endpoint responses were acquired into private temporary
storage outside Git on 2026-11-02. Native validation produced **965 ordinary
forms/pools and 303 supported moves**. The assembled envelope SHA-256 was
`3c65dc0c09e30db1c025fd93c49345575b51d4dff0a6cbe8a8f38273ee064cf3`.
The dated identifier is a local source label, not a remote Git revision.

| Artifact      | SHA-256                                                            |
| ------------- | ------------------------------------------------------------------ |
| Stats         | `5d5ac4679d0c9acc3d2ba0d0b520be18326ffc2bdb5fbc5dbc03848ca911ebd1` |
| Types         | `298d1f24fb162e496bfadad97e3cd0713bc1400f2606ce64e42a91ccb2ddc54a` |
| CPM           | `0a191dbf6fd414be6d91b7f7d2b875470f1439c5e8ee9f29326934af5b530fae` |
| Fast moves    | `68c034fe21d53aad75974993623a6e943a9b975ab6d39792d07e5f66cddfc43f` |
| Charged moves | `102575f0bbb187cec9f1a8c4cd3c05995d77da90c770f0cc39af946bbfc94ffc` |
| Current pools | `661b6a7347b5000a862379888a3750625f8430bea3be54547f06491d8f4d4462` |
| Type chart    | `459410ff68af897c130773eacb9d0df6a246af5f8998403f57d4d3f24550c695` |

Twelve hypothetical attack profiles across Dex 6, 68, 130 and 143 matched an
independent exact-BigInt rational checker for per-hit flooring, energy ceilings,
cycle timing, favorable/resisted type groups and dual-type vulnerabilities.
Source hashes were verified first. These are calculation examples, not observed
player Pokémon or measured battles. Raw/derived sample records and the private
checker report are not committed. Original fictional fixtures cover production
normalization, partial/ambiguous moves, external comparisons and confirmation.

## Executed checks and local Windows build

Completed checks for this increment: `npm run check` passed formatting, lint,
strict TypeScript, **152 frontend tests** and **8 adapter tests**. Native
all-target/reference-tools tests passed **172**, with **2 authorized-screenshot
tests ignored**. Production native reference CLI tests passed **2**. Initial
sandbox attempts at backup/fixture atomic renames failed on filesystem permissions;
the unchanged checks passed outside that sandbox. An independent static review
found no actionable defects; its move-response ordering concern was corrected
and covered by a regression test. This is not a measured battle-performance or
real-image OCR accuracy claim.

Rust formatting/check and the Windows executable/NSIS build passed. The ignored
local output is `src-tauri/target/release/bundle/nsis/Trainer Collection Manager_0.1.0_x64-setup.exe`
(12,318,065 bytes), SHA-256:

```text
329C3752412BABA01E4C9737EE0481B586E3CE96193D333DC95AAAB1DC321FF6
```

This is a local preview build, not a published release. The installer wizard and
installed-app real-data journey were not executed for this increment.

## Remaining work

Validate broader real gameplay scenarios before claiming combat accuracy. Add
gym defense next, then Rocket and conservative transfer-candidate review. Costs,
evolution eligibility, move acquisition, release restrictions, exceptional forms,
Shadow/status mechanics and actual readiness remain unsupported where stated.
Name/CP/IV OCR benchmarking and installed-app real-data acceptance remain open;
extra-field OCR stays deferred by the user's scope.
