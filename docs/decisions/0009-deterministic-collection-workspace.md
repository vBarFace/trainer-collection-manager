# 0009: Deterministic collection recommendations

Date: 2026-11-01. Current MVP supersedes ADR 0007's proposed ML milestones.

Use confirmed observations, installed local reference facts, actual collection
comparisons, explicit retention constraints and versioned Rust rules. No learned
model, LLM, cloud API or telemetry is introduced. The isolated utility-1 contract
remains intact; production reuses its readiness and deterministic mode vocabulary
and uses a narrower collection-1 DTO for the supported statistical scope.

Capture all evaluation inputs in a consistent native SQLite transaction. Context
hashes include observations/revisions, identity/reference context, applied tags,
optional details/evidence invalidation, latest lifecycle request tokens, reference
package digest, ruleset, and explicit global preferences with a revision. A real
Power Up can stale a plan even when no CP was supplied. Preferences preserve
unknown budget versus zero and change/revert invalidation through revisions.

Show collection decisions and statistical league potential separately from battle
readiness and investment. A complete supplied half-level table limits the modeled
ceiling to 40 or 50. Missing mechanics or incompatible observations abstain.
Retention constraints beat conditional evolution suggestions. Unknown costs never
pass affordability checks. No disposal recommendation is emitted in this scope.

Migration 11 reserves one current and one previous per-UUID plan, bounded reports
and progress. Recalculation rotates them atomically and never carries progress
silently. Progress is revision/context checked. Review completion is a request
for explicit observation confirmation; only compare/wait planning steps may be
locally completed without game evidence. Appraisal and confirmed action history
keep their independent contracts.

Existing confirmed evolution, Power Up and screenshot re-observation remain the
only corresponding observation-changing workflows. Navigation carries the
selected UUID into Inventory; no similarity matching or second lifecycle engine
is added. Native service tests validate restart, stale context, revision conflicts,
protection, preferences, reference changes and history-only Power Up.
