# Product requirements document

## Vision

Trainer Collection Manager is a free, open-source, offline-first desktop app for casual Pokémon GO players to maintain a local collection and receive explainable support for manual in-game decisions.

## Foundation scope

This release provides a Windows Tauri shell, private local inventory, screenshot review, confirmed saved CP/IV/identity corrections and deterministic appraisal evaluation. Users select one local PNG/JPEG, inspect a temporary preview and local name/CP/IV suggestions, enter/correct supported data fields, then explicitly confirm saving. Review confirms data only and creates a pending record with a new UUID. Users can select a saved Pokémon, correct supported observations and request an appraisal evaluation or re-evaluation separately. Inventory preserves existing records, UUIDs and assigned tags, and presents Pending, Evaluated, Needs data or Stale appraisal states after restart. Broad screenshot recognition, strategic recommendations, automatic tagging, account integration, teams, simulations, local LLM runtime, datasets, telemetry and cloud capabilities are deferred.

## Product principles

- Local-first and useful without AI or a network connection.
- User is always in control: recommendations are explainable and actions occur manually in Pokémon GO.
- Persist structured, reviewed data only. Never permanently retain screenshots or delete originals without explicit consent.
- Prefer deterministic, versioned formulas and simulation where appropriate.

## Intended import and evaluation flow

Select a screenshot → attempt local name/CP/IV extraction → review/correct the data → confirm saving → select the pending record → request appraisal evaluation → review facts, findings and limitations. Name, visible CP and IVs are suggested automatically for supported layouts; unreadable or hidden fields remain unresolved and can be entered manually. Experimental geometry matches are tentative suggestions, validated on one supplied real example, with no general accuracy measurement. Review confirmation does not assert that the Pokémon has been evaluated. Existing user-assigned tags do not establish evaluation status.

### New import and existing-individual management

Ordinary screenshot import adds newly acquired individuals after review and confirmation. It creates a new stable UUID and never associates, merges, replaces or updates an existing individual by species, CP, IVs or similarity. Existing validation and atomic saving remain in force.

Managing an existing individual starts by selecting its known UUID and choosing an explicit supported action. Current actions correct CP, appraisal IVs or species/form data; they update that same individual only after confirmation. A data correction does not establish that an evolution or Power Up happened.

Future lifecycle actions follow: select individual → choose action → perform it manually in Pokémon GO → explicitly confirm the result → update the same UUID and record the confirmed lifecycle change. An optional action-specific screenshot must supply observations for that selected UUID, rather than enter ordinary import. Preserve lifecycle history when those actions are implemented; S2.1 introduces no lifecycle-history storage or screenshot-update flow.

For example, a future confirmed Charmander → Charmeleon evolution must use validated reference evolution relationships, retain the individual's UUID, update the confirmed species/form, preserve IVs where mechanics permit, and replace affected CP/moves with newly confirmed values or explicit unknowns. Never retain obsolete pre-evolution values as confirmed post-evolution observations. An available evolution does not prove it occurred, and the app never performs in-game actions.

## Saved CP correction (S2.1)

- Select a saved Pokémon and choose **Correct saved CP**. The draft loads its saved CP. Enter a positive whole number or leave the field empty to mark CP unknown, then explicitly confirm. **Cancel CP correction** discards the draft without persistence; selecting another individual also discards it.
- Correction can set, replace, clear or restore CP. Rust validates the full SQLite positive integer range without a game-specific maximum; decimal strings preserve large values exactly. Saving never infers level, Power Up count, resources spent, moves, evolution or strategic usefulness.
- An actual correction preserves UUID, name, IVs, identity, tags, creation time and appraisal results/lifecycle. It updates the modification timestamp and an internal persisted strategic observation revision. CP correction does not run or invalidate appraisal.
- Submission locks while saving. Recoverable failures retain the draft. If CP or any tracked confirmed observation changed since the form opened, saving fails safely and requires an authoritative inventory reload before another attempt. A change followed by a revert still conflicts. Missing/deleted records also require reload. Successful saves display the native-confirmed value.
- Strategic observation revision is independent of appraisal revision and advances on actual CP, IV or confirmed species/form text changes, including clearing/adding unknown observations. No-ops, reads, appraisal runs, labels, tags and dataset/provenance refreshes do not advance it. It detects same-second changes and persists after restart. It supports future invalidation only; no strategic assessment or recommendation is calculated.

## Appraisal evaluation

- The offline Rust engine uses confirmed appraisal IVs only. All three values are required; unknown values remain unknown and produce Needs data with explicit missing fields.
- Known values yield attack, defense and HP IVs (0–15), total appraisal IVs (0–45), and appraisal percentage (`total × 100 / 45`). A 15/15/15 appraisal produces a noteworthy perfect-appraisal finding.
- These facts describe appraisal only. They do not establish PvP rank, PvE strength, overall quality, transfer suitability or universal competitive superiority. PvP, PvE, gym, Rocket and investment analysis have not been performed.
- Each evaluation attempt has a ruleset version and UTC timestamp. Changes to actual IV values or a ruleset mismatch make it Stale; re-evaluation replaces the attempt metadata. Name, CP and tag changes do not affect this scope. Stale results are withheld until re-evaluation.
- Findings, future recommendations/suggested tags and assigned tags are distinct. This engine emits no recommendations and never changes tags or infers shiny status or user preferences.
- Selection, IV correction and evaluation controls support keyboard use, loading, safe errors and retry. Full inventory editing and filtering remain deferred.

## Saved appraisal-IV correction

- Select a saved record and choose **Correct saved IVs**. The form starts with the confirmed values. Supply all three integers from 0 to 15, or clear all three to mark the group unknown, then explicitly confirm. Partial groups are rejected; cancelling or switching records discards the unsaved draft.
- Only IVs can be changed. Names, CP, UUIDs, creation timestamps and tags are preserved. Changed IVs invalidate a prior appraisal through the existing lifecycle; unchanged values preserve its state. Saving never runs evaluation automatically.
- Save failures retain the draft for retry. If current saved IVs differ from those displayed when correction began, reject the save and require a reload before retrying that draft. Missing/deleted records also require reload. No screenshot selection or extraction is needed for correction.

## Reference identity foundation

Task 006 separates manually confirmed species/form identity from display names and tentative OCR. A selected record exposes reference resolution, available metadata and version/provenance, with a narrow correction form. Missing or ambiguous identity remains explicit; unknown form never implies a default. UUIDs, observations, tags and appraisal lifecycle are preserved. The catalog loader validates versioned offline snapshots, but this release bundles an empty catalog pending verified redistribution rights. No real GO metadata or strategic recommendation is claimed. See [reference data](../development/reference-data.md).

## Utility classification and progression roadmaps — proposed

Task 008 specifies future behavior; S1 implements isolated fictional contracts and S2.1 adds saved CP correction and persisted observation revision only. No strategic evaluation, trained recommendation model, roadmap storage or progress UI is implemented. The appraisal and correction workflows above remain the implemented baseline. See [decision 0007](../decisions/0007-individual-progression-roadmaps.md) and the dependency-ordered [delivery backlog](../development/roadmap.md#proposed-delivery-backlog).

### Roles, actions and readiness

Each analyzed individual has a current-usefulness assessment, zero or more supported current and potential roles, a suggested summary tag where justified, and an explainable roadmap for the user's selected goal. Insufficient evidence produces an explicit unknown assessment and missing-input guidance rather than a guessed role or investment. Support multiple roles without forcing one exclusive class or a universal quality score.

| Concept                       | Meaning and proposed behavior                                                                                                                                                                                                                                       |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Current usefulness            | Explain what this individual can do with its confirmed present identity, moves and stats in a named context, including limitations and evidence.                                                                                                                    |
| Current roles                 | Supported uses now: `ATTACKER`, `GYM DEF`, and later `ROCKET`, `PVP 1500`, `PVP 2500`. A role is distinct from its readiness.                                                                                                                                       |
| Potential roles               | Uses achievable after specified, feasible preparation; identify the resulting form and conditions. A potential role is not a claim of current usefulness.                                                                                                           |
| Readiness per role/context    | **Usable now**, **Usable with limitations**, **Preparation required**, or **Insufficient data**. State the limitation or required preparation. This is separate from appraisal state and plan validity.                                                             |
| Suggested next action         | First unfulfilled, feasible preparation action on the selected path, such as `EVOLVE` or `POWER UP`. Move changes are structured actions with no new default tag. If only an untagged action is appropriate, show that instruction and leave the action tag absent. |
| Suggested summary tag         | Show the next-action tag when applicable; otherwise a supported ready role such as `ATTACKER` or `GYM DEF`. Keep its action/role kind explicit. With insufficient evidence, show no tag and explain what is needed.                                                 |
| Assigned tags and protections | User-owned collection annotations, independent of suggestions. `SAVE/SPECIAL` means retain; explicit user protection constraints also gate plans. Custom tags have no inferred strategic meaning.                                                                   |

Keep `TRANSFER` and `SAVE/SPECIAL` and all existing vocabulary available without implementing every classification now. Use `EVOLVE`, never a duplicate `EVOLUTION` tag. Appraisal percentage, observed CP, a role suggestion or an assigned tag alone cannot establish usefulness, authorize disposal, or justify an investment. Protect retained individuals from conflicting disposal proposals; irreversible evolution also requires the user's stated permission to change that individual. Recommendations never assign/remove tags, rewrite observations or perform in-game actions. Applying any suggestion as an inventory annotation requires a separate explicit user operation in a later increment.

### Individual roadmap contract

The roadmap belongs to the inventory Pokémon's stable UUID, including after a user-confirmed evolution. Start with one selected goal/active roadmap per individual; show other roles and alternatives within that assessment. Do not require team planning or a general workflow engine.

- Record the goal, intended role and context (for example, a specified encounter and conditions), current assessment/readiness, confirmed inputs, assumptions, missing inputs, user constraints and resource budget. A zero budget differs from an unknown budget.
- Provide ordered steps, explicit prerequisites and required/optional status. Each step has an action, reason, measurable completion evidence and limitations. Evolution names an exact target species/form in its dataset namespace. Unknown eligibility remains unknown.
- Move changes name required or optional moves, availability conditions and applicable acquisition constraints. Targets state a justified level, CP range or other measurable goal, calculation/rules version and contextual assumptions. If a required target cannot be justified, request data instead of presenting a ready investment plan. A CP or IV threshold alone never implies universal usefulness.
- Show estimated costs by resource, their source/version and uncertainty, or explicit unknowns. Unknown costs cannot pass a hard budget check; cost estimates are not actual spending. Explain expected benefit against use now and cheaper feasible paths, including limitations. Never invent game levels, moves, costs or performance numbers.
- Support a short ordered list per alternative, with explicit conditions such as obtaining a required resource or move. Select one path; mutually exclusive alternatives do not accumulate costs. Conditions must be satisfied before their dependent step is actionable. Include **use now** when supported and **wait / collect missing data** when no investment is justified.
- Associate the suggested next-action tag with the selected path. Retain observation revision and minimal input snapshot, goal/resource/protection revision, ruleset version, every dataset namespace/version/content digest, and model version/digest or explicit deterministic-only mode. Record generation time, explanation and uncertainty. See [proposed data model](../architecture/data-model.md#proposed-strategic-assessments-and-roadmaps-task-008).

### Progress and re-evaluation

The app has no game-account connection. A suggestion, **I completed this action**, and a verified observation change are distinct. Here, verified means validated structured values explicitly confirmed by the user; it never means independently verified against the game. A checkbox alone cannot update species, stats, moves, remaining resources or actual costs.

| Reported action | Minimum follow-up confirmation                                                                                                                                                                           |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Evolution       | Confirm resulting species and form for the same UUID, review resulting observed CP and moves, and confirm any level used by the plan or leave it unknown. An unexpected target invalidates the old path. |
| Power-up        | Confirm new observed CP and level evidence required by the target; if level is unresolved, keep target completion unverified. Do not infer new stats solely from the action count.                       |
| Move change     | Confirm actual fast/charged moves and relevant second-attack state; recheck availability constraints. A completed action does not prove the intended move was obtained.                                  |

After any action, resources spent/remaining require separate confirmation if needed for another budget decision. Preserve IVs unless explicitly corrected. Missing follow-up fields stay unknown; dependent steps remain blocked. A user can correct or undo an action report without silently reverting saved observations.

Observation corrections (including identity, CP, IVs, moves, level and relevant individual properties) make affected strategic plans **stale**, even for a change/revert. Goal, resource or protection changes and dataset/ruleset/model updates also require re-evaluation when used by the plan. Unsupported versions or known impossible paths are **invalid**. A plan is **blocked** when required facts or a verified budget are missing, or an unmet prerequisite prevents its next required action from safely proceeding. Ordinary future dependencies block their dependent steps, not an otherwise feasible plan: an evolve-first path can suggest `EVOLVE` while its later power-up step awaits evolution. A stale/invalid plan is historical context, with its actionable suggestion withheld. A blocked plan shows the reason and only safe data/preparation guidance. Review the changed inputs and explicitly re-evaluate; a replacement plan explains the change and requires fresh confirmation of carried progress.

The appraisal lifecycle remains independent: only actual IV changes or appraisal-rules changes invalidate `appraisal-1`. Strategic observation revisions must not reuse `appraisal_revision`. Re-evaluating a roadmap neither runs appraisal nor changes tags. Retain the current plan and one preceding plan with minimal inputs and progress notes, rather than unlimited history; details and conflict handling are in the [data model](../architecture/data-model.md#proposed-strategic-assessments-and-roadmaps-task-008).

### Fictional acceptance scenarios

These are contract scenarios, not validated Pokémon GO conclusions. All identities and targets below are placeholders; they assert no real species, move, cost or performance fact.

| Scenario                | Assessment and suggested tag                                                                                             | Roadmap and observable outcome                                                                                                                                                                                                        |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A: evolution required   | Fictional form A is preparation required for a potential `ATTACKER` role; suggest `EVOLVE`.                              | Confirm eligibility → evolve to explicitly identified fictional form B → obtain suitable moves if required → power up to an evidence-backed target if required → use as `ATTACKER`. Unknown prerequisites block the dependent action. |
| B: usable now           | Fictional individual is usable now as `ATTACKER`; the default no-investment path summarizes `ATTACKER`.                  | Present use now alongside optional further investment. Suggest `POWER UP` only if the user selects an improvement path whose contextual benefit and cost justify it.                                                                  |
| C: preparation required | Fictional individual is below its goal, with potential `ATTACKER` or `GYM DEF` roles; selected path suggests `POWER UP`. | Reach a justified target → use in the selected role. Explain each role/context separately and never treat one target as universal.                                                                                                    |
| D: insufficient data    | Missing identity, moves or required mechanics; no strategic tag.                                                         | Show missing fields and a blocked assessment. Marking an evolution done without confirming its result does not advance dependent steps or prove usefulness.                                                                           |
| E: changed plan         | A saved plan's budget, input or dataset changes.                                                                         | Withhold the outdated suggestion; preserve action reports, explain the mismatch, and explicitly regenerate. Reject a stale progress save rather than overwriting newer progress.                                                      |

Deliver the first real vertical slice for **ATTACKER in one explicitly bounded PvE context**, conditional on reviewed game data and mechanics. This provides a measurable current-versus-prepared comparison. `GYM DEF` follows because it also requires a defensible definition of defense utility and assumptions about opponent, time horizon and gym conditions. An attacker slice still needs move/combat data, costs and validated calculations absent from schema 1; reference identity alone does not unblock it. A fictional slice may validate contracts and UI, never strategic accuracy.

## Future constraints

Each Pokémon instance must have a stable UUID. Standard tags are `EVOLVE`, `POWER UP`, `ATTACKER`, `GYM DEF`, `ROCKET`, `PVP 1500`, `PVP 2500`, `TRANSFER`, and `SAVE/SPECIAL`; custom tags are allowed. IV appraisal and second attacks are fields/features, not default tags.

Future strategic recommendations require validated species/form identification, explicit structured moves/level and other applicable observations, licensed and versioned game data, validated formulas, and user goals where relevant. They must have their own scope and provenance. CP or aggregate IV percentage alone must never justify transfer, evolution or power-up.

The intended engine combines deterministic rules with a small model trained by this project, only after successful evaluation. Deterministic validation, feasibility, protection and sequencing remain authoritative; the first proposed learning objective is narrowly scoped role suitability. Training data, labels, baseline, independent evaluation and offline fallback are defined in the [evaluation plan](../development/evaluation.md#proposed-strategic-and-learning-evaluation-task-008). Screenshot/OCR benchmarking remains a separate concern, with no default screenshot retention or automatic training contribution.
