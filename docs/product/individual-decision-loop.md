# Individual decision loop

User-confirmed product objective, 2026-11-02. This defines the intended complete
experience; it is not a claim that every acceptance case is implemented.
Implementation boundaries are recorded in
[the preparation planner](../development/mvp-preparation-planner.md).

## Outcome

After confirming a screenshot import, the user should immediately understand
which uses are supported for this individual, what is worth preserving and what
to do next. Evaluation combines confirmed CP/appraisal IVs, identity, optional
manual facts, imported external references, suitable deterministic models and
the actual owned collection. Owned alternatives personalize the decision; they
do not replace external evaluation of species/form and move usefulness.

Keep the individual UUID throughout its lifetime. Evolution, Power Up, attack
changes and additional observations continue this individual's decision loop.

## Recommendations and suggested tags

- Assess gym attack, gym defense, Rocket, each supported PvP league, collection
  value and transfer review independently. An individual may have several uses,
  one use, collection value only, or insufficient evidence for a decision.
- Suggest every justified role tag together. Do not force a single winning tag,
  use the selected global goal as a filter that hides other useful roles, or
  produce a universal quality score. Explain why each suggestion applies.
- Separate current usefulness from future usefulness after preparation. A role
  supported only for a future evolution must name that evolution and its
  prerequisites; it must not portray the current form as battle-ready.
- Separate suggested tags from user-applied tags. Reanalysis updates suggestions
  without silently applying/removing the user's assignments or protections.
- Distinguish evidence of Shiny/size/other properties, personal collection value
  and statistical rarity. Size comparisons need an appropriate species/form
  reference or explicit evidence; raw weight/height alone must not trigger an
  arbitrary global rarity claim. Missing rarity evidence does not mean no value.
- Preserve protected/favorite/special individuals and meaningful future uses in
  transfer decisions. Low CP, low IV percentage or weakness in one mode alone
  cannot establish that disposal is appropriate.

## Collection-wide reanalysis and retirement review

Provide an explicit **Reevaluate entire collection** action. Recompute the
supported assessments against one consistent snapshot of the collection, current
facts, goals, resources and active reference versions. Present a review of what
changed since the previous assessment: new/lost/uncertain suggested roles,
investment priorities, roadmap changes and individuals worth reviewing for
retirement. Include reasons and relevant alternative individuals. Distinguish
changed collection priority from changed reference-based performance.

Having another individual of the same species, form or role is not a transfer
reason by itself. There is no default one-per-species or one-per-role limit.
Several attackers can contribute through team depth, coverage, different attacks,
availability, reserves or another useful role. Collection and special-property
value remain independent reasons to keep an individual.

Keep **ready current usefulness** separate from **unrealized future potential**.
A promising new individual receives its preparation roadmap without automatically
retiring a useful existing one. Evaluate the actual remaining investment and
requirements of the proposed replacement. Even after it is prepared, reevaluate
the older individual's remaining contribution rather than assuming replacement
is one-for-one.

Distinguish keeping in active use, keeping as a reserve, stopping further
investment, preparing another individual and reviewing for transfer. Reduced
investment priority does not imply no retention value. A transfer-review
suggestion needs evidence that the individual no longer contributes to the
relevant collection goals, with protections and unresolved roles made explicit.
The application never performs an in-game transfer or silently changes applied
tags, protections or observed facts during reanalysis.

Preserve the previous assessment and roadmap context so the user can understand
the change. Missing/stale evidence or unavailable sources must not be treated as
proof of retirement. Reanalysis itself neither spends nor reserves resources.

### Consistency is required for collection-wide review

User-confirmed on 2026-11-02: repeating an assessment must not keep generating
different advice when its evidence has not changed.

- The same confirmed collection, observations, constraints, reference contents
  and ruleset versions must yield the same semantic recommendations and plans,
  including after reopening the application. Reassessment does not add randomness.
- Equivalent options need stable deterministic tie-breaks. Database/provider
  iteration order must not decide which individual or attack is recommended.
- A changed result needs an attributable cause: a relevant individual/action,
  new observation, changed constraints/resources, updated references or a new
  version of a decision rule. Preserve before/after evidence and explain it.
- Compare decisions using stable individual/role/plan identities. Ordering,
  timestamps or reworded text alone do not create a meaningful "changed" alert.
- Define and version role-specific decision margins before switching investment
  or retirement recommendations on marginal differences. Ties alone cannot
  justify replacement or transfer; surface equivalence/uncertainty instead.
- Regression cases must cover repeated unchanged assessments, restart,
  unchanged-value saves, input ordering/ties, and a real change followed by
  repeated stable reassessment of that new state.

The current native engine already fingerprints and snapshots its inputs. The
`collection-5` / `roles-3` follow-up fixes an order-dependent Trainer-move tie and adds native
repeat/restart regression coverage. The future collection change-review UI and
its material-change policy remain to be implemented.

## One progression plan with several possible roles

The roadmap follows the individual through supported evolution stages and level/
move changes. It should explain a next justified action, the intended future
state, target level/projected CP when supported, prerequisites, costs, move
acquisition method and when to stop investing. Retaining the current state or
waiting remains a valid plan.

Plan action order from the intended role(s) and available mechanics. Compare
Power Up before/after evolution when supported; do not assume that evolving
requires reaching a particular CP or hard-code one species' example as a general
recipe. Imported requirements determine evolution eligibility. Target CP is a
projection of a supported state, not a universal evolution trigger.

Several useful roles may share a preparation path. Where they conflict, show
the tradeoff before an irreversible action. For example, a gym investment that
would exceed a chosen PvP CP limit must not silently discard that PvP option.
Suggest preserving an alternative individual or choosing a branch when justified.
Do not concatenate incompatible single-role plans into one checklist.

## Reanalysis after confirmed events

After confirmed import, new optional observations, an actual confirmed evolution/
Power Up/move update, relevant collection changes or reference changes:

1. Evaluate the current observed state and remaining future opportunities.
2. Refresh suggested roles/tags, preservation reasons and next actions.
3. Revalidate or replace the roadmap using the new evidence and constraints.
4. Preserve UUID/history and distinguish past suggestions from the current plan.

The user confirms actions they actually performed in the game. Reporting an
action must not invent its new CP, attacks or consumed resources. Show any
reconfirmation needed for fresh estimates. Make reanalysis part of this normal
flow rather than requiring the user to discover a separate analysis screen and
manually regenerate a stale plan every time.

OCR remains name, CP and appraisal IVs only. Moves, weight, height, variants and
other supported facts remain optional manual observations. Missing fields may
limit a specific recommendation or exact cost, but do not prevent saving or a
supported limited assessment.

## Acceptance scenarios

- **Several future uses:** given fictional evidence that an individual Charmander's
  future form supports gym attack and Ultra League, show both potential roles
  with the future form identified. Produce compatible preparation alternatives
  with CP/level/cost and move conditions. This is a product scenario, not a claim
  that Charmander or any particular specimen is competitively strong.
- **Evolution stages:** after the user confirms Charmander → Charmeleon, preserve
  its UUID, update its identity and reevaluate the remaining path toward any
  supported Charizard role. New CP/moves stay unknown until observed or otherwise
  supported; the old instructions do not remain actionable blindly.
- **Special collection value:** retain a confirmed Shiny or evidenced special-size
  specimen when collection value warrants it even if combat investment is weak.
  Explain the preservation reason without inventing a global rarity rate.
- **Several simultaneous roles:** allow attacker + defender, attacker + Rocket,
  or another supported combination. Collection-only individuals do not need a
  fabricated combat recommendation to justify keeping them.
- **Promising newcomer:** keep a useful ready Charizard while giving a newly
  caught Charmander its own conditional roadmap. Higher future potential alone
  does not justify a transfer recommendation for the Charizard. Assess both
  again after the newcomer is genuinely prepared, allowing both to remain useful.
- **Several individuals in one role:** retain two supported attackers when both
  contribute to the chosen team/coverage/reserve needs. Do not manufacture a
  defender role to justify retaining the second attacker.
- **Collection changes:** after importing new individuals, bulk reanalysis shows
  which existing suggestions or roadmaps changed and why. A justified retirement
  review preserves uncertainty, safeguards and the user's final decision.

## Gap from the current implementation

The current preview has independent limited role signals, conditional preparation
alternatives, cost/acquisition support, selected-path persistence and stale-plan
safeguards. It does not yet satisfy this whole loop: paths are generated for one
selected goal, there is no complete evidence-backed multi-role tag policy or
cross-role path optimizer, import/actions do not automatically refresh the full
recommendation/roadmap experience, and size-based collection rarity is not
implemented. Existing collection snapshot calculation is not yet the dedicated
collection-wide change-review and contribution-based retirement workflow above.
Current PvP stat/move signals do not establish competitive viability.

The next cohesive development increment is the integrated individual decision
loop: justified multi-role suggestions, compatible staged plans, automatic
reanalysis through the existing confirmed lifecycle, and collection-wide change
review with contribution-based retirement safeguards. This extends the existing
engine and roadmap; it does not replace them or broaden screenshot OCR.
