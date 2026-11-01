# Local reference adapter coverage

Reviewed: 2026-11-01. These adapters accept user-selected local JSON files. They do not download data, bundle provider records, establish redistribution rights, or assert complete game coverage. The three adapters are alternatives for the active snapshot; this increment does **not** merge their fields into a combined catalog.

## Supported file shapes

- **PvPoke:** `src/data/gamemaster/pokemon.json` (an array), or `gamemaster.json` with a `pokemon` array. Read `dex`, `speciesId`, `speciesName`, `baseStats.atk/def/hp`, and `types`. `hp` becomes GO base Stamina, never observed HP. The second-slot `none` type is removed. Named parenthetical forms, Shadow/Mega IDs, and ambiguous ordinary entries sharing a Dex number are withheld. This is a bounded adapter, not a replacement for Task 007's explicitly reviewed mappings.
- **PokeMiners:** `latest/latest.json`, an array of templates. The adapter expects a `data` object (or a directly flattened template), `pokemonSettings`, `stats.baseAttack/baseDefense/baseStamina`, `type/type2`, and `playerLevel.cpMultiplier`. Only an exact `V<dex>_POKEMON_<pokemonId>` template match with no nondefault explicit form is accepted. Source enum names supply labels; exceptional display-name spellings are not independently normalized. Integer-level CP multipliers are retained without inventing half-level interpolation.
- **PoGoAPI:** either separate raw response files selected together or a local JSON envelope containing `pokemon_stats`, `pokemon_types`, and optionally `cp_multiplier`, each copied from the corresponding public endpoint. Join explicit `Normal` forms by `pokemon_id`; rows with omitted forms are not assumed Normal. Read `pokemon_name`, `base_attack/base_defense/base_stamina`, `type`, and CP `{level,multiplier}` records. Values above level 40 are withheld because provider documentation warns about uncertain interpolation above that range.

Example envelope structure (no real dataset records):

```json
{
  "pokemon_stats": [],
  "pokemon_types": [],
  "cp_multiplier": []
}
```

Empty arrays do not make an installable catalog. Users need a nonempty supported source. The application must show errors and keep the previous snapshot when structural validation fails.

## Field-level matrix

“Available upstream” describes a candidate source, not implemented support or verified completeness. The strongest useful source varies by field; no universal provider precedence is applied.

| Field                                         | Current normalized support                                                         | Strongest candidate / follow-up                                                                                   |
| --------------------------------------------- | ---------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| National Dex and ordinary identity            | All three adapters; explicit user confirmation still required                      | Reviewed mappings for exceptional names and forms; retain provider namespaces                                     |
| Regional, named, costume, Mega forms          | Unsupported; withheld                                                              | Explicit mappings using GM form settings plus provider IDs; never suffix guessing                                 |
| Primary/secondary types                       | All three, nullable when absent                                                    | GM configuration, cross-check normalized providers                                                                |
| GO base Attack/Defense/Stamina                | All three; required for accepted rows                                              | GM configuration plus independently checked normalized records                                                    |
| Normal evolution links and prerequisites      | Unknown (`null`), including apparent terminal forms                                | GM evolution branches; PoGoAPI evolution endpoints provide selected requirements; verify special cases            |
| Fast/charged move identities and availability | Unsupported in normalized package                                                  | GM move pools plus PvPoke and PoGoAPI current/elite/legacy pools; model acquisition restrictions separately       |
| PvP move damage/energy/turns/buffs            | Unsupported                                                                        | PvPoke `moves.json` supplies PvP fields; GM combat move settings for cross-check                                  |
| PvE move damage/energy/duration               | Unsupported                                                                        | GM move settings or PoGoAPI `fast_moves.json`/`charged_moves.json`; never reuse PvP move power                    |
| CP multipliers                                | PoGoAPI supplied rows up to 40; GM integer rows up to 50; none from PvPoke adapter | Independently verify exact values and applicable level range before broader investment claims                     |
| Half-level and trainer/buddy level rules      | Only supplied PoGoAPI half-level values through 40; other rules unsupported        | Versioned mechanics contract and independent numerical examples                                                   |
| Type effectiveness                            | Unsupported                                                                        | PoGoAPI `type_effectiveness.json`, cross-check GM combat configuration                                            |
| Power Up/Candy XL/Stardust costs              | Unsupported, never zero                                                            | GM upgrades and PoGoAPI powerup requirements, including individual modifiers                                      |
| Evolution costs                               | Unsupported, never zero                                                            | GM branches and PoGoAPI evolution requirements, including conditional requirements                                |
| Shadow/Purified/Mega mechanics                | Unsupported                                                                        | Separately versioned GM mechanics with independently verified formulas; individual status is not species identity |
| Release/event availability                    | Unsupported                                                                        | Time-stamped provider release/event data; currentness and form context required                                   |
| Global rarity/encounter probability           | Unsupported                                                                        | None of these imports establishes reliable current encounter probabilities                                        |
| PvP rankings/simulations                      | Unsupported                                                                        | Explicit PvPoke league/cup/shield/rules assumptions, snapshot freshness and compatible moves required             |

Collection uniqueness can be derived from confirmed inventory identity. It is distinct from global rarity. A stat-product calculation, if enabled by another module, is distinct from a battle ranking. No complete PvE/Rocket/gym evaluation follows from the catalog above.

## Identity and provenance boundaries

Each adapter uses `local-<provider>-normal-1`, species `dex-<number>` and form `dex-<number>-normal`. These identities are separate from Task 007 mapped datasets. Updating an active source cannot authorize a silent association change. Switching providers requires existing pinned individuals to remain unavailable until the user explicitly reconfirms identity.

The package records provider, user-supplied upstream revision, retrieval date, raw-file SHA-256, source/rights URLs, notice, limitations, and normalized catalog version. A user-entered revision is a declaration; the application has not verified that it matches a Git commit. SHA-256 identifies the exact imported bytes. The PoGoAPI envelope digest identifies the combined local envelope, not separate endpoint responses; separately selected endpoint files now retain individual raw SHA-256, endpoint URL, user-declared revision and local acquisition date in an artifacts manifest. The original upstream download date and remote commit match are not verified. Cross-provider field merging remains a follow-up.

## Rights and source evidence

- **PvPoke:** [repository](https://github.com/pvpoke/pvpoke), [MIT software license](https://github.com/pvpoke/pvpoke/blob/master/LICENSE), [ordinary data fields](https://raw.githubusercontent.com/pvpoke/pvpoke/master/src/data/gamemaster/pokemon.json), [PvP move fields](https://raw.githubusercontent.com/pvpoke/pvpoke/master/src/data/gamemaster/moves.json). Software MIT licensing is not independently verified authority for underlying game-data redistribution. Current web inspection confirmed field shapes; no downloaded real records were committed.
- **PokeMiners:** [repository and disclaimer](https://github.com/PokeMiners/game_masters#disclaimer), [obfuscation warning](https://github.com/PokeMiners/game_masters#note-on-obfuscation). The maintainer attributes ownership to The Pokémon Company/Niantic and states an educational purpose. No explicit downstream dataset redistribution grant was verified. Field names can change or be uncertain. The initial browsing service could not retrieve the large raw file. A subsequent explicitly approved bounded local acquisition successfully validated the current pinned 19.5 MB snapshot; see the real-reference validation record. This establishes bounded structural compatibility, not broad factual/mechanics accuracy or redistribution rights.
- **PoGoAPI:** [documentation](https://pogoapi.net/documentation/), [legal notice](https://pogoapi.net/Legal/), [CP multiplier endpoint](https://pogoapi.net/api/v1/cp_multiplier.json), [evolution endpoint](https://pogoapi.net/api/v1/pokemon_evolutions.json). Local caching advice supports the intended technical access pattern but is not an underlying-data redistribution grant. The legal notice invokes fair use and disclaims endorsement/responsibility. Some documentation examples and gameplay descriptions are old; inspect actual versioned bytes before treating them as current mechanics.

Original project code remains MIT. User-controlled local acquisition does not resolve all rights questions. Do not commit provider files, normalized real snapshots, screenshots or game assets, or copy them into release bundles. Existing [source research](reference-data-sources.md) and [third-party rights review](../legal/third-party-assets.md) remain relevant.

## Executed adapter validation

`cargo test --manifest-path src-tauri/Cargo.toml source_adapter --lib` executed on Windows: **4 passed, 0 failed**. Original fictional inputs cover PoGoAPI explicit Normal form joining and upper CPM bound, PvPoke variant/ambiguity withholding, GM template identity/integer multipliers, and malformed/unsupported sources. `rustfmt --edition 2021 src-tauri/src/source_adapter.rs` also executed.

These fictional tests establish bounded structural behavior. Subsequent current real-source validation and independent hypothetical numerical checks are recorded in [the real-reference evidence](mvp-real-reference-validation.md). Broad factual/mechanics accuracy, OCR accuracy, legal authorization and competitive performance remain outside these claims.
