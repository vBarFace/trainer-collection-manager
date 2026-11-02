# Individual roadmaps and confirmed progress

## Next role-assessment requirements

The user reaffirmed that individual roadmaps remain central to the upcoming gym,
Rocket and transfer increments. Each increment must connect its reference-based
role assessment and personalized priority to supported next steps for the same
UUID. Lead with broad opponent-type guidance, distinguish the individual's typing
from actual/proposed move types, and show a few opponent examples when useful;
exact matchups are optional detail. Preserve unknowns, action confirmation and
stale-plan checks. See the [current priority contract](mvp-current-priorities.md).
These are requirements for upcoming work, not additional implemented actions.

## Implemented collection/statistical preview

The first [gym-attack increment](mvp-gym-attack.md) extends this preview with the
same UUID's proposed attack profile, optional observed-move review and an external
reference-comparison decision. Only the existing review/compare/wait confirmation
semantics are used; no investment or battle-completion action is fabricated.

Open Recommendations, choose an individual assessment, and create its personalized
roadmap. Inventory links to the same UUID's assessment. The roadmap covers the
currently supported collection/statistical goal: missing-data review, comparison
with owned alternatives, and a wait/no-investment alternative. It does not invent
executable evolution, move changes or Power Up targets when mechanics/costs are
unknown.

Ordered steps show required/optional status, prerequisites, rationale, nullable
cost and executable state. **Postpone**, **Ignore** and reset are planning choices;
ignoring a required review never establishes evidence or unlocks an investment.
**I completed this** on a review records `review_requested` and opens the selected
UUID's Inventory confirmation workspace. Use its existing identity, IV, CP,
optional observation, evolution, Power Up or screenshot confirmation as relevant.
Only local compare/wait planning steps can be directly marked completed.

Neither progress nor a reported action fabricates species, CP, level, moves or
remaining resources. Confirmed observations and action history are authoritative.
Native context/revision checks reject stale or concurrent writes. Changes to any
relevant observation, collection alternative, constraint, preference, reference
package/ruleset or lifecycle event withhold old actionable progress. This includes
Power Up with unchanged CP. Recalculation preserves the preceding report/progress
as historical context and starts fresh progress. Same-context generation is
idempotent. Restart retains both bounded slots and their evidence context.

The initial six native service integration tests and six recommendation/roadmap frontend
tests passed. These cover confirmation navigation, stale controls, context and
progress conflicts, failed reload/retry, persistence, reference/preference changes
and retention. Full gameplay investment plans remain gated on supported mechanics,
eligibility, moves and known costs; a visible wait state is intentional.
