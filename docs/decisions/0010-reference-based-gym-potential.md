# ADR 0010: Reference-based gym attack potential

- Status: accepted for the bounded first gym increment
- Date: 2026-11-02

## Context

The user prioritizes gym attack/defense, Rocket and transfer review before expanded
PvP. Name, CP and appraisal IVs remain the only automatic screenshot fields;
other observations are manual and optional. Assessment must draw on external
references independently of owned alternatives, use understandable type groups,
and connect to the same individual's roadmap.

## Decision

Extend the existing local package with optional typed `combat` data. PoGoAPI's
PvE move records, explicit Normal-form current pools and complete supplied
attacking-to-defending type chart support this first model. Store artifacts and
digests under the existing bounded, transactional activation/rollback contract.
Keep schema-1 identity namespaces and immutable SQLite migrations unchanged.
Old packages without combat data load as before; old roadmap JSON defaults its
missing gym assessment to unknown. `collection-3` and `gym-attack-1` version the
new results and stale earlier plans.

Compute a fixed reference scenario, not an actual battle simulation:

- Equal hypothetical level for both sides; the CP multiplier cancels in Attack
  divided by Defense. Defender base Defense is 200 and Defense IV is 15.
- Use the individual's confirmed Attack IV, or explicitly hypothetical IV15.
  Do not infer current level from CP or present a modeled level as observed.
- Per-hit damage: `floor(power * Attack / (2 * Defense) * STAB * effectiveness) + 1`.
  STAB is 1.2 when the move type matches either attacker type; dual defender
  effectiveness multiplies the two supplied scalars. No PvP damage bonus applies.
- One fresh-energy cycle uses `ceil(charged cost / fast gain)` fast attacks and
  one charged attack. Divide their summed modeled damage by supplied total
  durations. Incoming attacks/energy, repeated-cycle carryover, survival, weather,
  friendship, motivation, dodging and animation/battle scheduling are not modeled.
- Withhold zero-power/status attacks, dynamic typing, special form mechanics and
  known Shadow individuals. Unsupported cost/acquisition never becomes zero or
  an executable investment.

Choose the best neutral cycle per charged-attack type for the displayed form,
showing at most three profiles. Favorable single-type groups require both a
supplied charged-type advantage and an improved whole-cycle estimate. Display
typing vulnerabilities as typing information, not a defender rating. Numerical
details and benchmark examples are expandable. The default view explains role,
coverage, proposed/confirmed move names, missing evidence and limitations.

Benchmark against compatible ordinary forms in the imported reference, including
unowned options, with hypothetical IV15 and the same scenario. Percentiles are
within that supplied subset and attack type, not overall game rankings or proof
that all forms are released/obtainable. An owned alternative can change investment
priority without changing this individual's reference calculation.

Normalize conservatively around observed provider inconsistencies. Positive-energy
records in the charged endpoint become fast only when an explicit current Normal
pool independently requests that exact name as fast. Exclude non-spending charged
records, powers above 1000 and ambiguous display names with differing move IDs.
Record unresolved/excluded pool options per form. Malformed fields, duplicate
IDs, wrong-kind links and incomplete type charts reject activation. Partial combat
endpoints leave a catalog-only package with an explicit limitation.

Capture the active package and resolve saved identity pins against its catalog
inside the same SQLite snapshot. This avoids mixing another instance's newly
activated mechanics with old in-memory base stats. Include package/rules versions,
facts, preferences, tags and lifecycle tokens in the existing plan fingerprint.

Add optional move review and reference-comparison steps to the existing UUID
roadmap. Review requests open manual confirmation; comparison completion records
only a planning decision. No checkbox changes moves, CP, resources or history.
No Power Up/evolution investment is emitted without verified prerequisites/costs.

## Evidence and consequences

The damage/STAB interpretation was checked against the primary
[PoGOBase implementation](https://github.com/JuliaPokemonGO/PoGOBase.jl/blob/0cc81f7b0954cf11bb04cd52f546a9673a8e5773/src/damage.jl),
and local pinned GM battle settings independently supply STAB 1.2. Provider shapes
are described by [PoGoAPI](https://pogoapi.net/documentation/); real bytes rather
than old prose examples were validated. The developer's
[battle-system update](https://pokemongo.com/news/battle-systems-update) confirms
that gym/raid damage, energy and duration rules can change; no fixed model is a
claim of full current-server simulation accuracy.

No new dependency, migration, application networking, model weight, real dataset
fixture or third-party asset is added. Source/game-data rights remain separate
from MIT code licensing. See [usage, validation and limitations](../development/mvp-gym-attack.md).
Gym defense, Rocket and transfer decisions remain subsequent increments.
