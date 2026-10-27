# 0007: Individual utility assessments and progression roadmaps

Date: 2026-10-27. Status: proposed implementation design (Task 008); no runtime, migration or model change.

## Decision

Separate current usefulness, multiple current/potential roles, per-role readiness, suggested next actions and user-assigned tags. Reuse `ATTACKER`, `GYM DEF`, `EVOLVE` and `POWER UP` with explicit role/action kinds; preserve remaining vocabulary. Unknown evidence yields abstention. A roadmap belongs to an inventory UUID and selected goal, and explains feasible required/optional steps, conditions, targets, costs, alternatives and versioned evidence. See the [PRD](../product/PRD.md#utility-classification-and-progression-roadmaps--proposed).

Keep authoritative strategic logic in pure native domain functions. Deterministic logic owns input validation, missing data, reviewed mechanics/costs/prerequisites, feasibility, hard budgets, protections, target calculation and sequencing. Descriptive schema-1 evolution strings are not executable rules. Any mechanics extension needs reviewed typed data and its own versioned contract.

A project-trained component may score role suitability or later rank already feasible alternatives. First learn only `ATTACKER` suitability for one bounded PvE goal from reviewed labels; compare a regularized logistic regression and small decision tree with the strongest simple deterministic baseline. Do not train a tag predictor or general planner first. Model output cannot introduce facts, targets or actions, bypass prerequisites or override protections. Validate all output; absent, incompatible, uncertain or failing inference uses the deterministic baseline or abstains. Runtime remains offline and optional. See [evaluation](../development/evaluation.md#proposed-strategic-and-learning-evaluation-task-008).

Use a separate strategic observation revision and goal/resource/protection revision, plus dataset, ruleset and optional model identity. Do not repurpose `appraisal_revision`; appraisal remains governed by decision 0005. Progress distinguishes suggested steps, user action reports and validated user-confirmed observations. No game-account verification exists. Changes make dependent plans stale; invalid versions or impossible paths invalidate them. Prerequisites block dependent steps; block the whole plan when missing evidence or constraints prevent its next required action from safely proceeding. Withhold outdated action suggestions and explicitly regenerate.

Propose native-owned storage of one current plan and one prior plan per individual, including minimal input snapshots and progress evidence. Re-evaluation atomically rotates the prior slot; do not automatically carry completed flags into changed steps. Expected revision checks prevent stale observation/progress writes. No event-sourcing system, generic workflow engine, screenshot retention or unlimited history. Exact schema/migration and typed IPC require a future implementation review; see [data model](../architecture/data-model.md#proposed-strategic-assessments-and-roadmaps-task-008).

## Alternatives and consequences

- One classification tag is compact but confuses a role with preparation and loses multiple uses; typed roles/actions with readiness explain both use now and improvement.
- A learned planner would need trustworthy path labels and could propose impossible steps. Deterministic generation makes feasibility inspectable; learning is a replaceable suitability/ranking layer.
- Recomputing everything without snapshots loses the context for user progress after updates. Current plus one prior plan preserves a useful comparison with bounded storage, at the cost of no complete history or exact replay of unavailable old datasets/models.
- A single global appraisal revision would invalidate unrelated IV facts and miss non-IV changes. Separate strategic revisions add a small contract but preserve existing appraisal semantics.

Deliver a bounded `ATTACKER` context first; add `GYM DEF` after defining and validating its defense objective. Real strategic accuracy remains gated on appropriate authorized data and validated mechanics, independently of Task 007's merged adapter. Fictional scenarios validate contracts only. Training and redistribution require separate evidence and explicit contribution choice, as recorded in [third-party data policy](../legal/third-party-assets.md#proposed-strategic-data-and-training-review-task-008).
