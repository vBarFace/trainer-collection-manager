# Integrated individual decisions and progression

Implemented increment: `collection-6`, `individual-decision-1`, `preparation-2`
and `integrated-planning-1`. It extends the existing inventory, native reference
models and roadmap; no new game datasets, dependencies, OCR fields, cloud calls
or game automation are introduced.

## User experience

After a confirmed import, the new individual is selected and its suggestions and
roadmap load automatically in Inventory. Confirmed identity/CP/IV corrections,
optional facts, tags, evolution and Power Up trigger a fresh individual review.
History-only Power Up also refreshes even when CP/revision does not change.
Recommendations can open the same integrated review. A global role choice is
never required to obtain advice.

Suggested uses distinguish current support, preparation potential, collection
value and insufficient evidence. Qualified current/future suggestions appear
together, with the relevant form named. Unresolved roles remain under review.
Suggestions do not apply/remove tags. Confirmed Shiny/protection, useful duplicate
individuals and potential future roles retain independent collection value.

The app proposes a preparation route automatically. Manual route choice is an
optional override; its validated dataset/form/role/level intent survives a
confirmed evolution when still reachable. Steps and costs are rebuilt, not copied
as completed. The user can return to the app's automatic proposal. Changing a
resource allocation preserves the route intent. The same UUID, IVs, user tags,
history and explicitly confirmed facts remain the anchor throughout the loop.

A failed derived review is shown separately from a successful import/action.
Retrying review never repeats the game confirmation. Older or mismatched async
responses do not replace the currently selected individual's advice.

## Evidence policy

Role suggestions are contextual heuristics with declared thresholds, not a
universal score or a competitive battle ranking:

- Attack: a supported set in the upper quarter of at least 10 imported same-type
  ordinary-form gym-cycle options. The fixed cycle model keeps its existing
  limitations. Fresh compatible moves/level/HP and consistent available CP
  evidence distinguish a supported current-use signal from preparation potential.
- Defense: upper-quarter natural bulk among at least 10 imported ordinary forms,
  supported typing and variant scope. Placement eligibility, motivation and actual
  defender timing remain conditional; bulk alone does not certify placement.
- Rocket: upper-quarter first-charge access among at least 10 same-type imported
  Trainer-Battle options, at least median charged efficiency, supporting stats and
  type coverage. Shields, opponent lineup and buff/variant effects remain explicit.
- PvP 1500/2500: stat product and energy-kit evidence are shown, while a strong
  competitive recommendation remains pending compatible matchup/meta evidence.
- Collection/transfer: preserve protected/special and unresolved contribution
  value. A duplicate or stronger newcomer alone never certifies retirement.

Future uses traverse bounded explicit evolution edges and name their future
forms. A qualified future form cannot promote the current form's weak raw signal
to a strong tag. Old raw-role recommendations remain explanatory evidence;
automatic EVOLVE/POWER UP suggestions come from the effective active route.

Screenshot display names can be nicknames. A limited review remains available
without canonical identity, but the app does not silently confirm a species/form.
All optional observations remain optional for saving; missing evidence limits the
specific decision or cost rather than inventing facts.

## Joint planning

All supported role contexts contribute alternatives. Qualified future forms and
explicit manual targets receive priority in the bounded display. Identical game
actions may support several qualified uses; their blockers are combined and
readiness is never strengthened by merging. Unsupported-role comparison routes
stay separate from qualified combined-use claims.

The automatic level checkpoint is the first supplied half level reaching 90% of
the same form's modeled level 40 output: neutral gym cycle, defensive bulk or
ordinary stat product as appropriate. This is an explicit preparation heuristic,
not a battle breakpoint or a claim of optimal spending. Great/Ultra alternatives
use their supported projected CP caps. Older comparison checkpoints remain
optional. Exact totals still require a supported starting level/cost profile.

Plans show projected CP and remaining Candy/XL/Stardust costs where supported.
They exclude same-form Power Up targets below confirmed observed CP, preserve
special/Elite attacks for review, and do not automatically replace an observed
set with a lower modeled neutral output. Evolution precedes further Power Up in
the staged route so the changed form/CP/attacks can be checked first.

CP-cap losses, replacement of a current-form league option, incompatible target
forms and differing move preparation are explicit conflicts. Applied PvP tags
and explicit league preferences are protected caps. Pending competitive value is
not silently discarded. Protected individuals receive no automatic irreversible
investment proposal. Active-route conflicts are upfront; other alternatives'
warnings remain available in expandable details.

The display is bounded to 16 alternatives; omitted supported future options are
identified as needing a separately verified route. This is not an exhaustive
cross-role/team optimizer. Collection-wide change review and retirement ranking,
competitive PvP models, special-size rarity and complete eligibility/event
coverage remain separate follow-up work.

## Native consistency and persistence

`individual_decision` captures collection evidence, generates through the context
CAS and reads back a fresh matching roadmap, with at most 3 conflict retries.
The returned report comes from that roadmap. Individual review builds joint plans
only for its target while retaining owned-comparison inputs. Repeated identical
input remains deterministic; source/model versions join the context hash.

Optional route intent uses the existing revisioned native resource settings.
Only context-checked select/reset commands change it. Automatic proposals never
become user choices. New report fields default when reading old saved roadmaps;
the new ruleset makes older contexts stale once. No database schema changes or
mutable migration edits are required. Structured backup includes confirmed user
settings/derived plans and still excludes source documents and screenshots.

## Executed validation — 2026-11-02

- `npm run check`: formatting, lint, strict TypeScript, 174 UI tests and 8 adapter
  tests passed. Coverage includes automatic selection/review after import,
  history-only Power Up, fact updates, failure isolation, stale async results,
  suggested/applied separation, active conflicts and return to automatic mode.
- Native all-target/reference-tools suite: 224 passed, 2 private-image tests ignored.
  Includes qualified multi-use/current/future decisions, weak-data abstention,
  meaningful duplicate retention, exact cost/CP examples, merged-role blockers,
  preserved CP caps, special moves/protection, repeat/restart consistency and
  automatic same-UUID review through two evolution stages with manual intent.
- Rust format/check and the 2 native adapter tests passed.
- Three hypothetical specimens using all 11 private source files passed native
  analysis and bounded roadmap persistence (965 ordinary forms). The branching
  specimen exposed repeated legacy future assessments exceeding the 128 KiB
  report bound; integrated future evidence is now stored once in the calibrated
  decision, retaining its typing/cost/conflict guidance. No data limit or test was
  weakened and no real source records were committed.
- `npm run tauri:build`: production frontend, native release and Windows x64 NSIS
  installer passed. Delivery artifact: 12,482,674 bytes; SHA-256
  `46c3ac2056879d0a79560235d788e57267fb1ed497636753297e8f28f750484e`.

Installed-app wizard and real-collection GUI acceptance have not been executed.
These checks verify software behavior and declared model arithmetic, not battle
outcomes or a complete competitive PvP/rarity model.
