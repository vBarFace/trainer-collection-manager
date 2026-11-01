# Deterministic collection scope

`collection-1` is a pure Rust ruleset. It reuses utility-1 deterministic mode and
readiness vocabulary without replacing lifecycle actions or appraisal-1.

Supported outputs are confirmed species/form collection uniqueness, same-form
owned alternatives, retention/review/wait guidance, and theoretical Great/Ultra/
Master League statistics when reference inputs support them. No universal quality
score, learned model, meta rank or executable spending/disposal action is emitted.

## Statistical formulas and limits

For GO base stats B, confirmed appraisal IVs I and a supplied CP multiplier m:

- CP = max(10, floor((B_attack + I_attack) × sqrt(B_defense + I_defense) ×
  sqrt(B_stamina + I_hp) × m² / 10)).
- Effective Attack/Defense = (base + IV) × m.
- HP = max(10, floor((B_stamina + I_hp) × m)).
- Stat product = effective Attack × effective Defense × HP.

These formulas were checked against the primary
[PvPoke implementation](https://github.com/pvpoke/pvpoke/blob/master/src/js/pokemon/Pokemon.js).
Tests use independently hand-worked **original fictional** numerical examples:
base 100/100/100, IV 0/0/0 and m=0.5 gives CP 250, HP 50, Attack/Defense 50,
and product 125,000. Base 65535 with IV15 and m=0.5 exercises integer overflow
boundaries: Attack/Defense/HP 32775 and CP 107420062.

Require a complete increasing supplied half-level table through level 50, or
explicitly limit the model to a complete table through 40. Ignore partial rows
above 40 if level 50 coverage is incomplete. Show the modeled ceiling with every
result. Level-40 IV ranks are not level-50 ranks. Exclude Best Buddy and exceptional
HP/temporary Mega/Primal mechanics. Missing/gapped tables abstain; GM integer-only
rows do not silently generate half-level mechanics. The current PoGoAPI adapter
withholds documented uncertain values above 40.

Compare all 4,096 theoretical IV spreads of the same form at their best modeled
level within the league CP cap. This is stat-product suitability, **not** battle
ranking or current readiness. Owned comparisons stay within the same form and
context. Observed over-cap CP excludes that league; a CP unexplained by modeled
inputs requests review rather than inventing the current level or powering down.
Missing moves, eligibility, costs and battle assumptions block investments.

## Honest unsupported states

PvE DPS/TDO, raid targets, gym defense simulations, Rocket matchups, meta ranking,
global rarity, encounter rates, move changes, second attacks, costs and resource
spending are not calculated. Normal evolution relationships from reviewed Task
007 snapshots can support conditional collection-entry review only. They do not
make evolution executable without prerequisites and permission.

Retention protects SAVE/SPECIAL and, at the service boundary, separately confirmed
protection/favorites/special variants. No transfer/trade candidate is emitted while
collection value remains unresolved. Suggested tags never become applied tags.

The 11 integrated Rust engine tests pass. They establish the bounded mathematical
and safety contracts, not broad gameplay accuracy. A representative real-source
compatibility run and independently reviewed real-species numerical examples are
still required for full gameplay-domain acceptance.
