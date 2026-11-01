# Real-source compatibility and independent numerical validation

Executed 2026-11-01 using explicitly approved, bounded HTTPS acquisition into
private temporary storage outside Git and release bundles. No raw or normalized
real dataset records are committed. Acquisition does not establish data-use or
redistribution rights.

## Successfully validated snapshots

All three inputs passed the production source adapter, schema-1 catalog and local
package validators through the offline `validate-source` developer executable.

| Provider               | Upstream revision                                     | Raw bytes                      | Ordinary forms normalized | CP multipliers                                 |
| ---------------------- | ----------------------------------------------------- | ------------------------------ | ------------------------- | ---------------------------------------------- |
| PvPoke                 | `04b2a39100e9a24085e7d9bf81b756e739a3b1a8`            | 1,365,802                      | 992                       | None in this adapter                           |
| PokeMiners Game Master | `8e227be44f288d34463e23bf04e9b564d3c16f79`            | 19,522,222                     | 1,022                     | 50 integer levels; no half-level interpolation |
| PoGoAPI                | `public-api-2026-11-01`, exact bytes identified below | 499,143 combined raw responses | 965 explicit Normal forms | 79 half levels, 1–40                           |

These counts describe the adapter's bounded accepted subset. They do not establish
species availability, complete forms, current balance accuracy, move legality or
encounter/league eligibility. Providers remain alternative active namespaces.

## Source fingerprints

| Artifact            | Source                                                                                                                                     | SHA-256                                                            |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------ |
| PvPoke pokemon.json | [Pinned source](https://raw.githubusercontent.com/pvpoke/pvpoke/04b2a39100e9a24085e7d9bf81b756e739a3b1a8/src/data/gamemaster/pokemon.json) | `4971a274d8a9f29267f87de2221e1c06389df0251d197dd543abf21b29432aee` |
| GM latest.json      | [Pinned source](https://raw.githubusercontent.com/PokeMiners/game_masters/8e227be44f288d34463e23bf04e9b564d3c16f79/latest/latest.json)     | `5c947ac64d1de8859bea1b3bf044609d74b6e8429d53f8cb1aa30a72a46dce84` |
| PoGoAPI stats       | [Endpoint](https://pogoapi.net/api/v1/pokemon_stats.json)                                                                                  | `5d5ac4679d0c9acc3d2ba0d0b520be18326ffc2bdb5fbc5dbc03848ca911ebd1` |
| PoGoAPI types       | [Endpoint](https://pogoapi.net/api/v1/pokemon_types.json)                                                                                  | `298d1f24fb162e496bfadad97e3cd0713bc1400f2606ce64e42a91ccb2ddc54a` |
| PoGoAPI CPM         | [Endpoint](https://pogoapi.net/api/v1/cp_multiplier.json)                                                                                  | `0a191dbf6fd414be6d91b7f7d2b875470f1439c5e8ee9f29326934af5b530fae` |

PoGoAPI responses were separately hashed; the deterministically assembled local
envelope digest was `3ab21957938c13d3d7a9d4b88703a214e50ec10b8c451afc7bde06e6e4cb3dbe`.
No remote Git revision is claimed for those mutable API endpoints.

## Independent numerical checks

Nine hypothetical normal-form statistical cases passed: three selected National
Dex entries (1, 63, 150), each under 1500, 2500 and uncapped CP, within the supplied
level-40 ceiling and hypothetical 15/15/15 IVs. Raw source hashes were checked
before use. Sample base stats agreed across all three providers.

The independent checker used exact rational arithmetic on the supplied decimal
CP multipliers. It squared the CP expression and used BigInt integer square root
for CP flooring, integer division for HP, and rational stat product, rather than
calling the Rust float implementation. CP, HP and modeled level matched exactly;
stat product agreed within a relative tolerance of 1e-10. This includes a sample
whose capped league optimum falls below level 40.

These are hypothetical calculation checks, not observed player Pokémon or claims
that every spread/level is legally obtainable. Current level, encounter floors,
variants, moves, meta performance, costs and actual battle readiness remain
separate unresolved inputs.

A subsequent independent ranking check enumerated all 4,096 IV spreads for each
of the same nine examples. It selected each spread's highest permitted half level
using an exact squared-CP inequality, then compared rational stat products by
cross-multiplication, including exact ties. All nine native rank and spread-count
results matched. This checks the sampled theoretical rankings through level 40;
it does not establish every species/form, level-50 mechanics, obtainable spreads,
eligibility or full PvP/PvE correctness. The checker and its private report remain
outside Git with the acquired source bytes; no source records are committed.

## Reproduce with private files

```powershell
cargo build --manifest-path src-tauri/Cargo.toml --features reference-tools --bin validate-source
src-tauri/target/debug/validate-source.exe pogoapi YOUR_SOURCE_ID YYYY-MM-DD STATS_JSON TYPES_JSON CPM_JSON
src-tauri/target/debug/validate-source.exe --sample-dex DEX_NUMBER pogoapi YOUR_SOURCE_ID YYYY-MM-DD STATS_JSON TYPES_JSON CPM_JSON
```

This tool never downloads data. It emits only provenance, counts and optional
clearly labelled hypothetical derived statistics, uses a disposable private
database for sampling, and never opens the actual application inventory. It is
feature-gated and excluded from the normal application/release bundle.

Executed after adding the tool: feature-gated native suite **160 passed, 2
authorized-sample tests ignored**; production reference CLI tests **2 passed**;
three expected CLI rejection checks passed (missing arguments, invalid sample
Dex, unsupported provider). Rust formatting and frontend formatting checks passed.

The existing [field/rights matrix](mvp-reference-coverage.md) still governs scope.
Earlier inability to retrieve the large GM through browsing tools is superseded
by this successful bounded local validation; underlying data rights remain
unresolved.
