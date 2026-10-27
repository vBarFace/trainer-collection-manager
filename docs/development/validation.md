# S2.3 confirmed Power Up and CP updates validation

Date: 2026-10-27. Base: `f404793` (verified against fetched `origin/main`). Dedicated branch: `feat/s2-3-confirmed-power-up`.

## Delivered behavior and boundaries

The existing individual detail view adds Power Up with optional manually entered CP, review preview, exact CP difference, explicit in-game confirmation and cancellation. Null updated CP means preserve the current observation; both unknown and known CP are retained without inference. The command updates one existing UUID and atomically records a distinct `power_up` event with before/after observations, manual CP provenance when supplied and native UTC confirmation time. It preserves identity/form, IVs, appraisal, tags, nickname and unrelated individuals. Request UUIDs reject duplicate confirmations, including history-only actions. Expected observation revisions reject stale drafts; retry preserves the request UUID.

Migration 8 extends the existing checked lifecycle kinds while retaining legacy event JSON, timestamps, rowids/order, foreign keys, index and immutability. Migrations 1–7 remain unchanged. The same bounded main-window permission registers the typed command. No dependency, gameplay recommendation, level estimate, resource model, game automation or screenshot pipeline was added. Existing UUID-scoped screenshot re-observation and ordinary new-individual import retain their contracts. There is no persisted spending or favorites field; existing tags are preserved.

## Executed checks

- `npm run test`: passed, 122 tests across nine files, including 21 Power Up renderer/integration cases and the new native bridge case. Sandbox Vitest could not rename temporary cache files; the approved outside-sandbox run passed.
- `npm run lint` and `npm run typecheck`: passed with the final integrated frontend.
- Prettier check of all changed/new supported source and documentation files: passed. Final `npm run check` still stops at 66 untouched baseline files with CRLF/LF differences; all later stages were run independently. No check was weakened.
- `npm run tauri:build`: passed, including TypeScript/Vite production build, optimized native executable and Windows NSIS installer (11.50 MiB). The sandbox attempt built the executable but could not resolve the NSIS download host; the approved outside-sandbox retry packaged successfully. The existing `.app` bundle-identifier advisory remains.
- `git diff --check`: passed.

- `cargo test --manifest-path src-tauri/Cargo.toml`: passed, 122 tests; two existing explicit-private-image tests ignored. Binary and doc-test targets passed with zero tests.
- `npm run rust:fmt` and `npm run rust:check`: passed. Existing incremental-cache hard-link warnings use copying instead.
- `npm run reference:test-native`: passed, two tests. The sandbox attempt encountered an `EPERM` temporary-file rename restriction; the approved outside-sandbox retry passed.
- `npm run reference:test`: passed, eight tests.
- The initial `npm run check` stopped at repository-wide Prettier failures. A read-only Prettier audit verified that all 77 reported tracked files differed from formatted output only in CRLF versus LF. No formatter policy or unrelated source content was changed. Changed frontend/document files are formatted separately; final integrated results are recorded above.

Native coverage includes explicit/missing/invalid wire fields, precision above JavaScript's safe-integer range, successful known/unknown CP updates, null-as-keep for known and unknown values, unconfirmed/invalid/stale/missing target rejection, same-second correction/revert conflicts, replay protection and cross-action request uniqueness, UUID/metadata/unrelated-individual preservation, rollback after history failure and revision overflow, retry after rollback, persisted history after reopen and migration preservation/rollback/cascade. Existing S2.1/S2.2 tests remain in the full suite. A separate read-only native review found no actionable defects.

Native GUI interaction and real Pokémon GO actions were not manually exercised. Automated renderer/native tests and a packaged build are distinct from a native runtime smoke test.

---

# S2.2 confirmed evolution and existing Pokémon re-observation validation

Date: 2026-10-27. Base: `8fffce8bb9cb0a135d24a0094de5a430dbf5b970`. Started with a clean checkout on `main`, then created `feat/s2-2-evolution-reobservation`. No commit, push, remote PR or gameplay action was performed.

## Delivered behavior and decisions

Selected individuals now expose explicit evolution target selection/confirmation, targeted screenshot review and confirmed history. Evolution validates the direct different-species edge and catalog version under an immediate transaction, retains the UUID/IVs/tags/nickname/appraisal and clears CP. Targeted review captures UUID/revision before selection, reuses the existing picker/extractor/OCR, shows tentative evidence and saved/proposed values, and applies only explicitly selected complete CP/IV patches. Missing fields keep their current values. Species/form mismatches block persistence; display labels never establish species. The new workflow never calls import or inserts an individual. Existing ordinary import continues creating new UUIDs.

Migration 7 adds one confirmed event table, index and update guard; migrations 1–6 are unchanged. Each event records request/individual IDs, before/after identity and pins, CP/IV values, source, reviewed identity, catalog context and UTC confirmation timestamp. All writes and event insertion commit or roll back together. Request UUID uniqueness rejects duplicate confirmation including no-op updates. Strategic observation revisions remain independent of appraisal and advance for actual observations only. Evolution has a guarded fallback increment for changed species pins with unchanged query text. No event-management framework, dependency or game dataset was added.

The native schema has no species-dependent moves, observed HP or level. Those fields were neither fabricated nor silently declared valid. Source images remain read-only and unretained; temporary OCR workers/previews are released. Historical fields without stored provenance are not retroactively relabelled. History covers these two workflows and follows the existing individual-delete cascade.

## Changed files and migration

- Renderer: new `src/LifecyclePanel.tsx` and `src/LifecyclePanel.test.tsx`; updated `src/Inventory.tsx`, `src/native.ts`, `src/native.test.ts`.
- Native domain/tests: new `src-tauri/src/inventory/lifecycle.rs` and `src-tauri/src/inventory/lifecycle/tests.rs`; updated `src-tauri/src/inventory.rs`, `src-tauri/src/inventory/strategic_tests.rs`, `src-tauri/src/reference.rs`.
- Persistence/IPC/ACL: migration 7 and its tests in `src-tauri/src/database.rs`; `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src-tauri/build.rs`, `src-tauri/permissions/inventory-review.toml`.
- Documentation: `README.md`, architecture overview/data model, PRD, roadmap, screenshot review, new `docs/development/lifecycle.md`, this validation log and local `docs/development/s2-2-pr.md`.
- Dependencies, lockfiles, production catalog and broad capabilities remain unchanged.

## Executed validation

- `npm test`: **100 passed across 8 suites**, including 17 lifecycle UI cases and exact IPC coverage. Includes confirmation/cancellation, branch selection, catalog unavailable, tentative opt-in, manual provenance, missing fields after evolution, species mismatch, pre-extraction revision capture, same-entry Inventory replacement, duplicate/lost-reply reload recovery, cleanup and stale history response rejection. Existing import and S2.1 tests pass. The sandbox first failed before execution on Vitest cache-rename `EPERM`; approved outside-sandbox runs passed, including the final full suite.
- `cargo test --manifest-path src-tauri/Cargo.toml`: **113 passed, 2 existing private-image diagnostics ignored**, with zero-test binary/doc targets also passing. Fifteen added native tests cover sequential named Charmander/Charmeleon/Charizard relationship fixtures, branching/invalid targets, read-only cancellation, preserved data, no duplicate inventory, missing patch semantics, exact identity review, stale/repeated confirmation, rollback on history insertion failure and revision overflow, catalog drift, equal-label species pins, persistence and migration failure/idempotence. Test relationship fixtures use artificial dex identifiers and no imported statistics or assets.
- `npm run lint`, `npm run typecheck`, `npm run rust:fmt`, `npm run rust:check`: passed. Existing Cargo incremental hard-link warnings fell back to copying.
- `npm run reference:test`: **8 passed**. `npm run reference:test-native`: **2 passed** on the approved outside-sandbox retry. Initial sandbox execution passed candidate validation but blocked a CLI temporary-file rename with `EPERM`.
- `npm run tauri:build`: passed on the approved outside-sandbox retry, producing the optimized Windows executable and **11.50 MiB x64 NSIS installer**. The sandbox compiled successfully but packaging could not resolve the NSIS download host (`11001`). The existing `.app` bundle-identifier advisory remains. The installer was not installed, and build output remains ignored. No native GUI smoke test was performed; UI/native behavior is verified by their automated suites.
- `npm run check`: stops at aggregate formatting. A read-only audit found **64 untouched files** differing from HEAD only by CRLF/LF endings; every normalized working file passed Prettier. Formatting this updated validation log removes one, leaving **63 unrelated pre-existing failures**. Changed supported files pass targeted Prettier, and `git diff --check` passes. Remaining aggregate stages were executed separately above.
- Independent read-only review identified and resolved two defects: equal-label/pin-only evolution did not initially invalidate old screenshot drafts, and repeated confirmations needed explicit reload recovery. Regression tests pass. Migration diff review confirms versions 1–6 were not edited.

## Limitations and S2.3

Production still bundles the empty reference catalog pending verified dataset rights. The evolution workflow therefore cannot validate real Pokémon targets in this release; no target is guessed. Release-reviewed schema-1 evolution edges and the existing adapter define normal permanent evolution; Mega targets are excluded by that adapter and same-species changes by the native action API. Trading, fusion and temporary transformations remain outside the supported contract. Screenshot identity is explicitly user-verified, not automatically proven, and recognition retains existing layout/confidence limitations.

Recommended S2.3: one selected player goal/context per individual with explicit unknown state, native persistence and its own narrowly scoped confirmation/revision contract. Tie later move/level/resource support to that goal; do not add strategic scoring or spending in the same slice.

# S2.1 saved CP and strategic observation revision validation

Date: 2026-10-27. Base commit: `3d89e3989ca187936aa3956c19d5f1dfe5bf5337`. Started from clean `main`, inspected the implemented S1 contracts/migrations, and created `feat/s2-1-saved-cp-correction` before editing. No commits, pushes, PRs, issues or remote changes were made.

## Delivered scope and decisions

Selected individuals offer **Correct saved CP** with positive/unknown values, explicit confirmation/cancellation, exact decimal strings, duplicate-submit protection, retained recoverable drafts and authoritative reload after conflict. The native result replaces the displayed record. CP correction preserves UUID, name, IVs, identity/pins, tags, creation time and appraisal metadata/results. It does not run appraisal or infer an in-game action, level, spending or usefulness.

Immutable migration 6 adds `strategic_observation_revision`, initially 0 for existing/new records. Migrations 1–5 were compared with HEAD and remain unchanged. Null-safe triggers track actual CP/grouped IV changes across full and narrow writes, and identity insertion/removal or changes to the exact confirmed species/form query tuple. Pins, catalog activation, provenance, labels, tags, appraisal and reads alone do not advance it. One Pokémon UPDATE changing CP and IVs consumes one strategic revision. An immediate transaction checks expected CP and revision before writing; change/revert and same-second changes invalidate old drafts. No-ops preserve timestamps/revision. Overflow aborts atomically, including any IV appraisal revision change. Existing IV/identity conflict contracts and appraisal invalidation remain intact.

Ordinary screenshot import still creates a new UUID after review, without matching, merging or updating existing individuals. Future evolution, Power Up and action-specific screenshots must operate on a selected known UUID and preserve confirmed lifecycle history. No such actions, strategic output, event log, dependencies, game data or goal/budget/protection revisions were added. **Only S2.1 is complete; S2 remains in progress.** The next recommended slice is S2.2: one explicit selected goal/context per individual, including unknown state and narrow persistence/concurrency, before selecting further observation fields.

## Files changed

| Area                         | Files                                                                                                                                                                                                                                                      |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Native persistence/contracts | `src-tauri/src/database.rs`, `src-tauri/src/inventory.rs`, `src-tauri/src/commands.rs`                                                                                                                                                                     |
| Registration/ACL             | `src-tauri/src/lib.rs`, `src-tauri/build.rs`, `src-tauri/permissions/inventory-review.toml`                                                                                                                                                                |
| UI/IPC                       | new `src/CpCorrectionPanel.tsx`, `src/Inventory.tsx`, `src/native.ts`, `src/review.ts`                                                                                                                                                                     |
| Tests                        | new `src-tauri/src/inventory/strategic_tests.rs`, new `src/native.test.ts`, `src/Inventory.test.tsx`; command/migration tests are in the native files above                                                                                                |
| Documentation                | `README.md`, `docs/architecture/data-model.md`, `docs/product/PRD.md`, `docs/decisions/0007-individual-progression-roadmaps.md`, `docs/development/roadmap.md`, `docs/development/screenshot-review.md`, `docs/development/utility-contracts.md`, this log |

## Executed checks

- `npm test`: final full suite passed, **82 tests in 7 suites**, including 7 Inventory CP cases and direct IPC precision coverage. Covers cancellation/focus, large strings, unknown/restoration, native-confirmed display, no-op submission, busy/duplicate guards, validation/retry, conflicts/deleted records, failed reload retry, change/revert revision reload, switching UUIDs and remounting from saved data. The sandbox failed before tests on a Vitest cache rename (`EPERM`); approved outside-sandbox runs passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: full suite passed, **98 passed, 2 existing private-image diagnostics ignored**; binary/doc targets contain zero tests. Eight new repository tests cover preservation, supported writes, provenance exclusions, two-connection conflicts, migration/reopen, same-second/change-revert, invalid/missing/deleted records and atomic overflow. Two new command tests cover strict fields, invalid values, precision and safe errors. After adding maximum-revision serialization assertions, the targeted `commands::tests::cp_ --lib` run passed all 3 selected tests. An initial full development run found four outdated latest-migration expectations; they and the v4 reconstruction were updated for migration 6, then the full suite passed.
- `npm run lint`, `npm run typecheck`, `npm run rust:fmt`, `npm run rust:check`: passed. Existing incremental-cache hard-link warnings fell back to copying.
- `npm run reference:test`: **8 passed**. `npm run reference:test-native`: **2 passed** on the approved outside-sandbox retry; the sandbox passed candidate validation but blocked the CLI temporary-file rename (`EPERM`).
- `npm run tauri:build`: passed on the approved outside-sandbox retry, producing the optimized executable and x64 NSIS installer (**11.47 MiB**). The sandbox compiled the application/frontend but could not resolve the NSIS download host (`11001`). The existing `.app` bundle-identifier advisory remains. The installer was not installed; ignored build output was not committed. No native GUI smoke test was performed; native/UI behavior was exercised by the separate automated suites above.
- `npm run check`: stopped at aggregate formatting. A fresh read-only audit verified all **60 initial failures** were untouched files differing from HEAD only by CRLF endings; every HEAD version and every working version normalized to LF in memory passed Prettier. Formatting this updated log leaves **59 unrelated pre-existing failures**. Unrelated files/formatter rules were not normalized. Remaining check stages were executed separately above. Targeted formatting for changed supported files and `git diff --check` passed.
- Scope review and independent read-only code review found no material issues. Dependency/lockfiles, production catalog and capability grants remain unchanged. The existing narrow permission adds only `correct_pokemon_cp`; migrations 1–5 are unchanged. Revision counters are absent from normal UI text, and no lifecycle action controls were added.

# S1 fictional utility contract validation

Date: 2026-10-27. Base commit: `0a365d145d7cc9c530e28b4314db46285ea4b8d7`. Started on `docs/task-008-utility-roadmaps` with the eight uncommitted Task 008 documents shown below already present. Created `feat/s1-fictional-utility-contracts` without discarding, staging or committing them. S1 adds separate contract/test/fixture files and this validation entry; the roadmap gains implementation status and the next bounded S2 task. No remote mutation or fresh external game-data collection was performed.

## Delivered scope and evidence

`src-tauri/src/utility.rs` contains `utility-1` types, explicit string precision/null semantics, bounded pure loading/validation and path/index errors. `src-tauri/src/lib.rs` adds only the public module declaration. `src/utility-contracts.ts` mirrors the wire fields without extending live native/inventory DTOs or IPC. Native tests in `src-tauri/src/utility/tests.rs` and renderer tests in `src/utility-contracts.test.ts` consume the same nine original fictional cases in `tests/fixtures/utility/acceptance.json`. Fixture notes and the [implemented wire contract](utility-contracts.md) document bounds and limitations.

SHA-256 comparisons against the starting working tree confirm Task 008's PRD, architecture overview, data model, evaluation plan, licensing document and ADR 0007 remain byte-for-byte unchanged. The original Task 008 validation section is also byte-for-byte intact beneath this new entry. Only its roadmap and validation file receive S1 updates. These preserved uncommitted documents remain visible in the overall diff against main and are not new S1 changes.

Cases cover evolution-first, use now with optional further investment, power-up-first, missing identity/moves/mechanics, zero budget, unknown budget/costs, protection conflict, stale context/report evidence and required move changes without a new default tag. Focused negative mutations exercise individual/goal associations, duplicate path/step IDs, missing/forward/self/cross-path/cyclic prerequisites, role/action mismatches, unknown versions, malformed revisions/amounts/digests, target ranges, bounds and contradictory suggestions. Structural validity is not evidence of real strategic accuracy.

Independent review found and resolved a current use-now summary bypass of missing evidence/step conditions and a Serde unit-model variant which discarded extra provenance fields. A targeted regression reproduced the model-field loss before the empty-struct variant fix. A separate regression reproduced Serde accepting object aliases for string enums; explicit string deserialization now matches renderer vocabulary. Active evolution targets also require their namespace in evaluated dataset provenance. These failed development regressions are distinct from final results below.

## Executed checks

- `npm run check`: stops at the same 58 pre-existing formatting failures outside the Task 008/S1 edit set. A fresh read-only audit verified every failure differs from HEAD only by line endings, all HEAD contents pass Prettier, and CRLF-to-LF normalization in memory passes. Unrelated files and formatter rules remain unchanged. The remaining stages were run separately.
- `npm run lint` and `npm run typecheck`: passed.
- `npm test`: all 74 tests in six suites passed on the approved outside-sandbox run (16 new renderer contract tests). The initial sandbox attempt failed before running any tests because Vitest's temporary-cache rename returned `EPERM`.
- `cargo test --manifest-path src-tauri/Cargo.toml`: final full suite passed, 88 tests passed and 2 existing private-image diagnostics ignored; binary/doc-test targets contain zero tests. Includes 23 new native contract tests. Earlier targeted runs validated fixture round-trips and the individual fixes.
- `npm run rust:fmt` and `npm run rust:check`: passed. Existing incremental-cache hard-link warnings fell back to copying.
- `npm run reference:test`: all 8 portable adapter/provenance tests passed.
- `npm run reference:test-native`: both tests passed on the approved outside-sandbox retry. The sandbox passed whole-candidate validation, then failed the CLI test on a temporary output rename (`EPERM`).
- Targeted Prettier passed for the 7 S1 TypeScript/JSON/Markdown files; all 31 local Markdown links/anchors passed; `git diff --check` passed. Rust formatting was checked separately. Scope checks confirm existing commands, inventory/appraisal/reference modules, renderer native DTOs, migrations, dependency/lockfiles, permissions and production catalog are unchanged from HEAD. The frontend production bundle contains no S1 contract/fixture strings.
- `npm run tauri:build`: passed on the approved outside-sandbox retry, producing the optimized executable and x64 NSIS installer (11.46 MiB). The sandbox compiled the application but failed resolving the normal NSIS packaging download host (`11001`). The existing `.app` bundle-identifier advisory remains. Ignored build output was not installed or committed. No native GUI smoke test was performed.

The earlier Task 008 validation remains historical evidence for that documentation-only increment. S1 adds no planner, strategic output, persistence/revision mutation, progress commands, UI, dependency, migration, model, capability or real dataset. Existing appraisal/inventory/tag/reference suites pass; the production catalog remains empty. The next bounded task is saved CP correction plus an independent strategic observation revision, as recorded under [S2](roadmap.md#s2--confirm-the-minimum-individual-and-goal-inputs).

# Task 008 utility and progression specification validation

Date: 2026-10-27. Branch: `docs/task-008-utility-roadmaps`. Scope: documentation only. The checkout was clean on `main` before branching. Remote GitHub branch reads confirmed the base and final main SHA remained `0a365d145d7cc9c530e28b4314db46285ea4b8d7`, containing merged PR #9. The main CI run was initially in progress; final inspection confirmed [run 40385059460](https://github.com/vBarFace/trainer-collection-manager/actions/runs/40385059460) completed successfully for that exact SHA. Remote inspection is separate from local checks below.

## Inspected implementation and proposal boundaries

Read repository guidance, PRD/roadmap, architecture/data model, appraisal/reference decisions and workflows, licensing/provenance research, validation scripts and CI. Cross-checked native inventory/IV revision and appraisal contracts, reference schema/activation, adapter provenance/ledger and renderer DTOs. Task 007 is integrated; production still uses `tcm-empty` / `empty-1`, and schema-1 evolution requirements are descriptive strings. Current appraisal emits no strategic recommendations. Moves, level, strategic revisions, roadmap storage and progress UI remain proposals.

Updated the existing PRD, architecture/data model, evaluation plan, licensing guidance and roadmap/backlog; added concise ADR 0007. Independent read-only review found an ambiguity that could block an entire feasible plan because a later prerequisite was unfinished. Corrected the PRD, ADR and data model: ordinary dependencies block their dependent steps; plan-level blocking concerns missing evidence/budget or inability to proceed safely with the next required action. No strategic accuracy is claimed from fictional scenarios.

## Executed local checks

- Targeted Prettier passed for all 8 changed/new documents; all 58 local Markdown links/anchors passed; `git diff --check` passed. Scope review confirmed all changes are under `docs/`.
- `npm run check`: the initial attempt stopped at formatting in 59 files outside the then-current edit set. A read-only audit verified all 59 differed from HEAD only by line endings, their HEAD contents passed Prettier, and their working contents passed after CRLF-to-LF normalization in memory. This validation log was subsequently updated and formatted; the final aggregate check still stops on the remaining 58 untouched files. Unrelated files and formatting rules were preserved. Later check stages were run separately.
- `npm run lint` and `npm run typecheck`: passed without warnings.
- `npm test`: 58 tests passed in five suites on the approved run outside the sandbox. The sandbox attempt failed before running tests because Vitest's temporary-cache rename returned `EPERM`.
- `npm run reference:test`: all 8 portable adapter/provenance tests passed.
- `npm run rust:fmt` and `npm run rust:check`: passed. Existing incremental-cache hard-link warnings fell back to copying.
- `npm run tauri:build`: passed on the approved outside-sandbox retry, producing the optimized Windows executable and x64 NSIS installer (11.46 MiB). The sandbox compiled the executable but packaging failed resolving the normal NSIS download host (`11001`). The existing `.app` bundle-identifier advisory remains. Ignored build output was not installed or committed.
- No Rust test suite, native reference integration test or GUI smoke test was newly executed for these documentation-only changes. Their implemented coverage was inspected; the remote CI result above is not presented as a local execution.

Changes are confined to documentation. No runtime code, migration, dependency, capability, production catalog, model, private image or global memory change was made. No external game data was collected, provider contacted, issue/PR created, push performed or remote setting changed. Remaining gates are dataset-specific rights, validated strategic facts/mechanics, confirmed input support and trustworthy training labels/evaluation. The next implementation task is [S1: fictional utility contracts](roadmap.md#s1--lock-the-first-utility-contract-with-fictional-scenarios).

# Task 007 adapter and compatibility validation

Date: 2026-10-26. Base: `4a13d22` (Task 006 PR #8), verified against fetched `origin/main`. Branch: `feat/task-007-reference-adapter`. The connected repository had no open pull requests at inspection. No real dataset or permissions request was submitted. [Source evidence](reference-data-sources.md#task-007-evidence-review--2026-10-26) records the rights blocker; the production catalog remains byte-for-byte unchanged.

## Results

- `npm run check`: stops at existing checkout CRLF/LF formatting differences. The first run reported 54 files, including three scoped docs not yet formatted. After formatting scoped docs, 50 untouched files remain. A read-only audit verified all remaining files differ from HEAD only by line endings, all committed texts pass Prettier, and normalizing working text in memory passes. No unrelated text or formatting rule was changed. Targeted Prettier checks of all changed/new supported files pass.
- `npm run lint` and `npm run typecheck`: passed without warnings.
- `npm test`: 58 passed in five suites, outside the sandbox. The sandbox attempt failed before running tests with Vitest cache-rename `EPERM`; the approved retry passed.
- `npm run reference:test`: 8 passed. Covers deterministic source/plan ordering, exact byte digests, schema drift, incomplete/invalid stats/types, rights/revision metadata, individual variants, explicit evolution knowledge, Mega rejection, compatible changes, historical identity tombstones, version/content mismatch and corrupt previous provenance.
- `npm run reference:test-native`: 2 passed outside the sandbox, using the actual Rust loader and offline CLI. Includes complete generated candidate validation, duplicate/dangling/cyclic/invalid/missing fields, repeated byte-identical generation, exact source/plan/output SHA-256, malformed input, missing input/history, no output after parse failure and refusal to overwrite output. Initial sandbox execution failed at a temporary output rename (`EPERM`); the approved retry passed.
- `npm run rust:fmt` and `npm run rust:check`: passed. Existing sandbox incremental hard-link warnings remain; Cargo copies files instead.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 65 passed, 2 existing private-image diagnostics ignored; binary/doc targets have zero tests. Includes stable pins through renamed/reassigned aliases, additions, missing metadata, removed/restored forms/species, cross-provider switches, conflicting IDs and failed activations. Inventory assertions preserve queries/pins, confirmation timestamps, UUIDs, observations, tags and persisted appraisal state.
- `npm run tauri:build`: passed outside the sandbox, producing the optimized Windows executable and x64 NSIS installer (11.46 MiB). The sandbox compiled the executable but could not resolve the NSIS packaging download host (`11001`); the approved retry packaged successfully. The existing `.app` bundle-identifier advisory remains. Build output is ignored and was not committed or installed. No new manual GUI smoke test is claimed.
- `git diff --check`: passed. No dependency/lockfile, migration, capability, production catalog, frontend behavior or IPC contract change.

The original fictional input's exact SHA-256 is `88f5e0696916cff6536c9077ca7d60125043d644e62471148d8c7d16abc80c33`; its plan is `83ca481113bd97412271306db7ff532e3d4b6fad45275758b5af67a4e9365b74`. Both CLI runs emitted catalog SHA-256 `cf7f3420641e16710cbc710a393a4f9b127e16136a0c0eec13f895cb0ab30342`. Source/plan byte digests depend on checkout line endings; normalized snapshot bytes do not. No real-data snapshot/digest is approved. No network is used by transformation or runtime; Cargo can need its normal development dependency cache.

Independent review found and corrected an inbound Mega evolution edge and insufficient previous-provenance digest checks; focused regressions pass. Structural checks cannot establish license authority, game correctness, semantic form reuse under unchanged IDs or an honest historical ledger. Reviewed evidence, cumulative release provenance and explicit mapping review remain release requirements. In-session native continuity history is intentionally not a persistent player-data migration. **The bounded real-data acceptance criterion remains blocked**, pending dataset-specific rights/provenance and per-record factual review.

# Task 006 reference-data validation

Date: 2026-10-26. Base: `cca38c9` on current `origin/main`, which includes Task 005 (PR #6) and confirmed-IV corrections (PR #7). Branch: `feat/task-006-reference-data`. The roadmap's next increment agrees with this scope; no conflicting Task 006 definition was found.

## Results

- `npm run check`: stops at repository-wide Prettier CRLF/LF differences. After formatting the scoped files, 40 untouched files remain. A read-only check verified each differs from HEAD only by line endings and its committed content passes Prettier. No unrelated file or quality configuration was changed.
- Targeted Prettier checks: all changed/new supported files pass.
- `npm run lint`, `npm run typecheck`: passed.
- `npm test`: 58 tests in five suites passed outside the sandbox. The initial sandbox run failed before running tests because Vitest could not rename a temporary cache file (`EPERM`); the approved retry passed.
- `npm run rust:fmt`, `npm run rust:check`: passed. Existing incremental-cache hard-link warnings remain; Cargo copies files instead.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 62 passed, 2 existing private-image diagnostics ignored; binary/doc targets have zero tests. The first incremental run caught one old schema-count assertion (4 instead of 5), corrected for the new migration.
- `npm run tauri:build`: passed, including Vite production output, optimized Windows executable and x64 NSIS installer (11.46 MiB). The initial sandbox attempt compiled the executable but could not resolve the NSIS download host; the approved retry completed packaging. The existing `.app` identifier advisory remains.
- `git diff --check`: passed.

## Coverage and review

Original fictional fixtures cover valid single/dual types, aliases/IDs, missing stats/evolutions, explicit null versus verified-empty lists, missing form, ambiguous names, unknown IDs, strict JSON/schema/provenance rejection, duplicate IDs/Dex numbers, dangling/cyclic evolution links and version handling. Failed activation retains the active dataset. A -> B -> reused A is rejected in-session; idempotent current-version activation is accepted.

Repository tests cover confirmed association, stable UUID/name/CP/IV/timestamp/tag/appraisal preservation, renamed aliases, unchanged confirmations retaining stable pins, conflict/missing-record handling, clear/delete cascade and unsuccessful catalog updates. A v4 upgrade/reopen test preserves the complete existing record and assignments; unresolved confirmed identity survives restart. Existing v1/v2/v3 migration and lifecycle tests also pass. Migration 5 conflict/rollback is covered. Narrow JSON input tests reject omitted nullable groups and unrelated fields. UI tests exercise metadata/provenance, unknown states, manual correction, clearing/cancellation, retained drafts, reload on conflicts, focus and shared action locking.

Independent review found and resolved two edge cases: unchanged confirmations after alias removal must preserve pins; previously active version identifiers must not accept different content after switching catalogs. Both have regression coverage.

## Offline evidence and limits

The native loader resolves compiled/local bytes and does not fetch source/provenance URLs. Tests load fictional source URLs under reserved `.invalid` domains successfully in the sandbox; the reference module contains no HTTP client or network call. New UI actions use local Tauri IPC only. Production embeds an empty manifest; fictional records are compiled only into tests. No runtime network dependency, dataset download, screenshot, personal inventory or artwork was added. No new instrumented full desktop network trace or manual GUI smoke test was run for Task 006; prior OCR runtime validation is historical evidence below.

The foundation validates structure and identity continuity, not game-data accuracy or license authenticity. Real GO metadata remains unavailable pending dataset-specific redistribution evidence. In-memory activation/version history is not durable; production restart reloads the bundled snapshot. Cross-release version immutability and reviewed upstream correctness require the future distribution procedure. No strategic formula, competitive ranking or recommendation is enabled. See [reference data](reference-data.md), [source research](reference-data-sources.md) and [decision 0006](../decisions/0006-versioned-local-reference-data.md).

# Saved-IV correction validation

Date: 2026-10-26. Scope: explicit correction of saved appraisal IVs, building on Task 005 commit `0b7b99b` on branch `feat/confirmed-iv-corrections`.

## Results

- `npm run check`: stops at the existing repository-wide Prettier CRLF/LF differences in 34 untouched files. A read-only diagnostic confirmed that all 34 differ from HEAD only by line endings and that their committed contents pass Prettier. The previously untouched `review.ts` is now changed and formatted within scope. No quality checks or configuration were weakened.
- Targeted Prettier: changed/new formatter-supported source and documentation files pass.
- `npm run lint`, `npm run typecheck`: passed without warnings.
- `npm test`: 40 tests in four suites passed outside the sandbox, including a final run after the form help update. The initial sandbox run failed before loading tests because of the existing Vitest temporary-cache rename `EPERM`; the approved retry ran the tests.
- `npm run rust:fmt`, `npm run rust:check`: passed. Existing incremental-cache hard-link warnings remain; Cargo copies files instead.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 53 passed; 2 existing private-image diagnostics ignored. Binary/doc targets contain zero tests.
- `npm run tauri:build`: passed outside the sandbox, producing the optimized Windows executable and x64 NSIS installer (11.43 MiB). The sandbox compiled the release executable but could not resolve the NSIS packaging download (11001); the approved final retry completed. The existing `.app` bundle-identifier advisory remains.
- `git diff --check`: passed. No migration, dependency, ruleset or screenshot-import contract change. The main-window command ACL adds only `correct_pokemon_ivs`.

## Evidence and limits

Native tests exercise explicit complete/null correction inputs, omitted/partial/fractional/negative/unrelated payload rejection, authoritative range validation, malformed/deleted records and safe conflicts. Repository tests cover adding unknown IVs, 0/15 boundaries, pending/stale/evaluated/needs-data transitions, correction persistence across reopen, metadata retention, unchanged submissions preserving timestamps/state, UUID/name/exact CP/creation-time/tag preservation, rejection of stale drafts and rollback of failed updates/revisions. A simulated intervening name/CP update is retained because correction SQL changes only IV columns.

Frontend tests cover prefilled values and focus, draft-only edits, cancellation, grouped validation, explicit confirmation before clearing to unknown, filling unknown groups, duplicate submission prevention, disabled controls during saving, stale results withheld after save, separate re-evaluation, retained drafts and retry after storage failure, conflict/deletion requiring discard/reload, fresh fields after remount or record selection, preserved CP/tags and ignored late completion after unmount. All existing import, OCR and appraisal tests remain intact.

A bounded independent code review reported no blocking issue. Its suggestion to explain draft loss when selecting another record was added to the form help.

The conflict check compares the saved IV group with the values shown when editing started inside an immediate transaction. It prevents overwriting different current IVs; it is not a revision-token contract and accepts an intervening change/revert restoring the expected group. The existing migration-4 trigger continues to govern appraisal invalidation. Full inventory editing, automatic evaluation, strategic recommendations and automatic tagging remain outside this increment.

No real collection records were modified by validation. A Windows build and automated renderer/native repository checks are not manual native GUI correction/restart verification; the installer was not installed or launched. No new dependencies, game data or assets were added, so no new licensing review was required. See [evaluation and correction workflow](evaluation.md).

# Task 005 appraisal evaluation validation (historical)

Date: 2026-10-26. Scope: persisted appraisal lifecycle, deterministic Rust rules and Inventory evaluation UI. Started from fetched `origin/main` at `b8cc42d` on `feat/deterministic-evaluation`.

## Results

- `npm run check`: stops at the repository-wide Prettier check because of existing CRLF/LF differences in 35 untouched files. The first run reported 37 files; the changed validation page and updated App assertion were subsequently formatted within scope. A read-only diagnostic verified every failing tracked file differed from its committed version only by line endings, and every committed version passed Prettier. No checks/configuration were weakened and unrelated files were not reformatted.
- Targeted Prettier checks: changed/new formatter-supported files pass.
- `npm run lint`: passed without warnings after moving the shared state labels out of the component module.
- `npm run typecheck`: passed.
- `npm test`: 32 tests in four suites passed outside the sandbox. The initial sandbox run could not load the suites because of the existing Vitest temporary-cache rename `EPERM`; the approved retry ran the actual tests.
- `npm run rust:fmt`, `npm run rust:check`: passed. Existing incremental-cache hard-link warnings remain; Cargo copies files instead.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 48 passed, 2 existing private-image diagnostics ignored; binary/doc targets have zero tests.
- `npm run tauri:build`: passed outside the sandbox, including the optimized Windows executable and x64 NSIS installer (11.43 MiB). The release executable compiled in the sandbox, but installer packaging initially failed resolving the NSIS download host (11001); the approved retry completed. The existing `.app` bundle-identifier advisory remains.
- `git diff --check`: passed. Existing migrations 1–3, dependency manifests/lockfiles, screenshot inputs and tag operations remain unchanged. The narrow command ACL adds only `evaluate_pokemon`.

## Regression evidence

- A frozen version-3 upgrade fixture models Task 004 databases. Migration/reopen tests preserve UUID, name, exact SQLite maximum CP, known IVs, creation/update timestamps, standard/custom tag identities and assignments. Existing version-1/version-2 upgrades and new-database initialization remain covered. A late migration-4 conflict rolls back the revision column and migration version.
- Repository tests cover Pending → Needs data, Needs data → Stale after adding IVs, Stale → Evaluated, clearing IVs, same-second change/revert, identical-value updates, and ruleset invalidation for known and unknown data. All four states and current results survive reopening. Name/CP/tag changes do not invalidate appraisal. Direct SQL IV updates also invalidate through the trigger.
- Failed first/repeated evaluation writes roll back safely; missing/deleted records return typed errors and deletion cascades appraisal metadata. Appraisal attempts retain UUIDs and Pokémon timestamps and never mutate user tag assignments.
- Pure engine tests exhaust all 4,096 valid IV combinations, including 0/0/0 and 15/15/15, repeat every evaluation deterministically, reject out-of-range inputs and assert that no recommendations are emitted. Unknown IVs yield explicit missing attack/defense/HP and no invented total, percentage or perfect flag.
- Typed command tests cover confirmed-input appraisal, serialized scope/version/state/facts, empty recommendations, unchanged assigned tags, malformed UUIDs and deleted Pokémon. Existing import confirmation, image/path payload rejection, extraction and persistence tests still pass.
- UI tests cover all four native states, empty/loading behavior, keyboard focus on selection, selected-ID-only requests, repeated-click prevention, native facts and limitations, findings/recommendations/assigned tags separation, retained stale state after failure, retry, missing data, reload after deletion, result rendering after remount and ignored late evaluation completion after unmount. Existing OCR/manual-review, cancellation, confirmation-only persistence and no-tag import tests remain intact.
- A bounded independent code review found no blocking correctness issue. It identified the stale empty-list assertion, a shared-constant Fast Refresh warning and review-specific storage error text; all were corrected.

## Limits and decisions

Automated renderer and native-command/repository checks plus a Windows build do not constitute manual native GUI evaluation/restart verification. The generated installer was not installed or launched as part of this task, and no test records were added to the user's real collection.

The current ruleset is `appraisal-1` and the scope is appraisal only. Only latest-attempt metadata and an IV revision are stored; current facts are recalculated. Stale result details are withheld because old inputs are not retained. No strategic evaluation, recommendation, automatic tag, shiny inference, game dataset, model or new dependency is introduced. There is no new licensing concern for this original-code increment. Future strategies require confirmed species/form and applicable moves/level, reviewed licensed/versioned game data, validated formulas and user goals where relevant. Existing OCR evidence remains one supplied example, without a representative accuracy benchmark.

See [evaluation workflow](evaluation.md), [data model](../architecture/data-model.md) and [decision 0005](../decisions/0005-persisted-appraisal-evaluation.md).

# Automatic OCR follow-up validation (historical)

Date: 2026-10-26. Supersedes the initial bounded milestone's name/CP deferral. Scope: bundled offline OCR, native crop preparation and real appraisal-bar support, with explicit editable confirmation.

- `npm run lint`, `npm run typecheck`: passed.
- `npm test`: 27 tests in four suites passed outside the sandbox, whose known Vitest temporary-cache rename restriction prevents loading tests. Coverage includes actual OCR-result admission, low/invalid scores, ambiguous/hidden CP, editable name/CP, exact confirmed payloads, cancellation during initialization, timeout, failures, unmount and late results.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 40 passed, 2 ignored private-reference diagnostics; binary/doc targets contain zero tests. Native additions cover real grey/orange/pink rounded/JPEG bars, all IV boundaries, contradictory rows, clipped HP anchors, metadata-free text crops, mixed-size CP normalization and overlays touching the final CP glyph.
- `npm run rust:fmt`, `npm run rust:check`: passed. Existing incremental-cache hard-link warnings remain.
- `npm run check`: stops at the verified pre-existing CRLF/LF formatting differences in 25 untouched files. The prior follow-up had 28; the three now-changed configuration files were formatted within scope. All changed/new formatter-supported files pass targeted checks. No quality check was weakened; generated third-party OCR assets are excluded alongside other generated/vendor assets.
- `npm run tauri:build`: passed outside the sandbox, producing a Windows x64 NSIS installer of 11,973,903 bytes. About 21.9 MB of uncompressed reviewed worker/core/language assets plus complete license notices are embedded. Existing `.app` bundle-identifier advisory remains.
- `git diff --check`: passed. Rust dependencies, migrations, inventory persistence, save fields, command ACL and native filesystem permissions remain unchanged. Only the existing screenshot response gains temporary OCR crops. CSP adds same-origin workers and WebAssembly compilation support.

## Real reference and worker verification

The user supplied and authorized local testing of one real portrait appraisal JPEG. Native IV extraction returned 15/2/2. Actual bundled Tesseract recognition on native-generated crops returned display name Abra (raw engine score 96) and CP 468 (score 73). These are uncalibrated engine scores, not measured accuracy percentages. Initial CP recognition failed admission; normalizing the small CP-prefix glyphs geometrically improved recognition without relaxing the score threshold or substituting characters.

The real browser worker was exercised in a headless Chrome against the application's actual TypeScript worker adapter and shipping Tauri CSP. Every external request was blocked; it returned the same five fields, made zero external requests and created zero IndexedDB databases. Observed OCR-only elapsed time was 248 ms in that run; this single environment/example is not a production-latency guarantee or accuracy benchmark. Loaded resources were the local worker, SIMD-LSTM core and English/Portuguese compressed language files.

The browser diagnostic initially hit the sandbox's local-network restriction (ERR_NETWORK_ACCESS_DENIED); the approved outside-sandbox run passed. A development native link attempt overlapped another test executable and hit LNK1104; rerunning sequentially passed. The private source was only read; no screenshot was committed, bundled or imported. One temporary derived CP crop used for visual debugging was removed. Normal tests require no private images.

Independent reviews found and helped correct clipped-green-run acceptance and the risk of an overlay erasing a final CP digit into a smaller plausible number. Both have regressions. Complete component notices were reviewed and bundled, including the worker's extracted notices, Tesseract/Leptonica and statically linked codecs. The worker is owned before initialization and terminated on every completion/failure/cancel/unmount/timeout path; language caching is disabled.

## Compiled Windows GUI smoke test

The newly built release executable was launched and inspected through Computer Use. Its native modal picker selected the authorized real JPEG; the actual Tauri/WebView2 review automatically populated Abra, CP 468, attack 15, defense 2 and HP 2, including both raw OCR engine scores. This confirms the bundled worker/resources and CSP work under `http://tauri.localhost/`, beyond the separate Chrome worker test. **Cancel review** removed the preview and displayed “Review cancelled. Nothing was saved.” The inventory remained empty. No confirmation/save was submitted; desktop GUI save/restart remains covered by automated contracts/reopen tests rather than claimed as manually exercised. The application was left open and ready for use, with no pending review.

General accuracy still needs a representative authorized labelled corpus and held-out evaluation. Unsupported layouts, dark themes, incomplete health bars, custom symbols and occlusion may remain unresolved or produce erroneous suggestions. Confirmation remains necessary; this increment does not evaluate or classify Pokemon. See [limitations](local-extraction.md) and [decision 0004](../decisions/0004-offline-hybrid-extraction.md).

# Task 004 initial bounded validation (historical)

Date: 2026-10-26. Scope: a bounded experimental local appraisal-bar extraction increment, typed temporary suggestions and editable review; **not general screenshot recognition**.

## Results

- `npm run check`: blocked at the repository-wide Prettier check by CRLF/LF differences in 28 untouched files in this checkout (`core.autocrlf=true`). Their committed contents pass Prettier, their working text differs only by line endings, and normalizing those endings in memory passes. The historical Task 003 count was 20; this checkout initially reported 39 files, before formatting only the files changed for Task 004. No checks/configuration were weakened and unrelated files were not reformatted.
- Targeted Prettier checks: all changed/new supported source and documentation files pass.
- `npm run lint`: passed without warnings.
- `npm run typecheck`: passed.
- `npm test`: 17 tests in two suites passed outside the sandbox. The sandbox attempt failed before loading either suite with the previously documented Vitest temporary-cache rename `EPERM`; the approved retry ran the actual tests. An initial implementation failure exposed IV descriptions accidentally included in labels; evidence is now outside each label, and the existing exact-label assertions pass.
- `npm run rust:fmt`: passed.
- `npm run rust:check`: passed. Existing incremental-cache hard-link warnings remain; Cargo copies files instead.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 32 passed; binary/doc targets have zero tests. The final debug suite reported 8.65 seconds; this is a whole-suite observation, not extraction latency or a production performance benchmark.
- `npm run tauri:build`: passed outside the sandbox, including TypeScript/Vite, optimized Windows executable and x64 NSIS installer. The sandbox compiled the executable but could not resolve the NSIS download host (11001); the approved run used the accessible packaging cache and completed. The existing `.app` bundle-identifier advisory remains. Installer output is ignored under `src-tauri/target/release/bundle/nsis/Trainer Collection Manager_0.1.0_x64-setup.exe`.
- `git diff --check`: passed. No dependency, lockfile, capability, permission, migration or production persistence change.

## Evidence and limits

New original in-memory geometric fixtures exercise all 16 IV values (including 0 and 15) at 240×300, 480×600, 960×1200 and 1920×2400; missing/unreadable tracks, crop boundaries, reversed fill, ambiguous fourth tracks, nonquantized and contradictory endpoints, transparent pixels, busy candidate limits, and a shared white overlay that would otherwise incorrectly rescale all three IVs. Requiring both separators fixes that demonstrated ambiguity. An intermediate fourth-track test was updated to draw the now-required separators, so it still tests four valid candidates rather than an unsupported solid rectangle.

The native PNG pipeline test verifies extraction before downsampling; a JPEG compression check requires either the known correct IVs or an unresolved group. This does not establish broad JPEG accuracy. Existing corrupt/unsupported images, resource limits, source immutability and metadata-stripping tests still pass. Existing grouped IV validation, exact CP, atomic rollback and reopen tests remain intact.

Frontend tests cover editable suggestions including zero, original provenance after correction, explicit confirmation only, retained corrections after a failed save, successful retry, unresolved extraction with successful manual fallback, cancellation without persistence, screenshot replacement clearing stale fields, duplicate submission prevention and preview release. A native extraction-to-confirmation test proves that extraction itself leaves a reopened database empty, rejects extra image/path/filename/extraction fields in save input, and persists only confirmed corrected IVs with unknown CP and no tags across reopen.

Name and CP recognition are **unavailable**. Tests assert their null/unavailable contract; they do not exercise OCR or claim recognition of clear/ambiguous/overlay-hidden text. Those text recognition requirements remain unsupported. No authorized labelled real screenshot corpus was available; no real screenshot recognition rate, calibrated confidence score, production latency or cross-platform runtime result is claimed. Synthetic rules may reject actual appraisal screenshots or match other graphics. Independent code review found the shared-overlay and label issues above; after correction, no additional actionable findings were reported by inspection.

The new installer was built but not installed/launched for a native GUI smoke test. Automated UI, native decoding/serialization and temporary-database reopen tests are not represented as end-to-end desktop runtime verification. No private screenshots are committed, and validation did not import test records into the user's actual collection.

The implemented bounded increment has no maintainer-approval blocker. Broader recognition requires authorized labelled examples and held-out evaluation; adding an OCR engine/model or changing Windows package identity requires a separate technical/distribution/license decision. See [decision 0003](../decisions/0003-bounded-local-extraction.md) and [limitations](local-extraction.md).

# Task 003 validation

Initial implementation results (before the data-only review follow-up below).

Date: 2026-10-25. Scope: local screenshot selection, manual review and confirmed inventory import.

## Automated checks

- `npm run check`: failed at the existing repository-wide Prettier check: 20 untouched files are checked out as CRLF (`core.autocrlf=true`), while the default formatter expects LF. For all 20, the committed HEAD text passes Prettier and the working text differs from formatted output only by CRLF/LF. No unrelated files were reformatted and no checks were weakened. Formatting of every changed/new supported source/document file passes separately.
- `npm run lint`: passed, zero warnings.
- `npm run typecheck`: passed.
- `npm test`: passed, 13 tests in 2 files. The sandbox attempt failed before loading either suite because Vitest could not rename a temporary cache file (`EPERM`); the approved run outside the sandbox passed.
- `npm run rust:fmt`: passed.
- `npm run rust:check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed, 24 tests; binary and doc-test targets contain zero tests.
- `npm run tauri:build`: passed outside the sandbox, including the final modal-picker change. Produced the release executable and `src-tauri/target/release/bundle/nsis/Trainer Collection Manager_0.1.0_x64-setup.exe` (1,856,051 bytes). Sandbox attempts could not access the external NSIS cache and failed during its download with DNS/host lookup error 11001; approved runs outside the sandbox packaged successfully.

Rust was installed in the user's `.cargo/bin` directory but absent from the shell PATH. Validation used that directory in the process PATH. The initial dependency fetch failed in the sandbox because `index.crates.io` could not resolve; the approved network retry downloaded dependencies successfully. Cargo emitted incremental-cache hard-link warnings and copied files instead; checks still exited successfully. Tauri also emitted its pre-existing advisory that the bundle identifier ends in `.app` (relevant to macOS naming); the Windows build target is unchanged.

The first Rust test run had 23 passes and one failure: `accepts_png_and_jpeg_content_and_returns_pixel_only_png` expected a 12-pixel preview width but received 800. `DynamicImage::thumbnail` enlarged small images. The implementation now resizes only when a side exceeds 1600 pixels. The unchanged assertion and the complete 24-test suite then passed.

Frontend coverage includes unknown/complete/all six partial IV groups, exact CP and Unicode name boundaries, explicit confirm/cancel, selected standard/custom tags, repeated submit while saving, recoverable failure with retained values and retry, persisted record rendering, safe errors and loading retry, preview replacement/cancellation/unmount, and ignored late picker results. Keyboard focus restoration after successful save is also asserted.

Native coverage includes content sniffing/full decode, unsupported/fake/corrupt images, input byte limits, dimension/pixel/output-buffer limits, metadata stripping, unchanged originals, safe error messages, exact signed-64-bit CP boundaries, typed grouped IVs, atomic record/tag rollback after missing-tag and SQL-trigger failures, independent UUIDs, and persistence across repository reopen.

## Native runtime smoke test

Partially verified in the compiled release using the Computer Use skill. The application opened and loaded the real local inventory through its native command. The native PNG/JPEG picker opened, and selecting a synthetic 400×600 PNG displayed the preview beside the clearly labelled manual form, with all supported fields and nine standard tags. No save confirmation was submitted by the smoke test.

The first picker had no parent window. The final implementation attaches it to the calling Tauri window, giving it normal modal ownership; native format/check/tests and the installer build were rerun successfully after this change. Further desktop automation was stopped when the helper reported mismatched control geometry, repeated user-input detection and a screenshot inconsistent with the target accessibility tree. Native GUI save, process restart and retrieval were therefore **not manually verified**. Their repository/command persistence and renderer behavior are covered by automated tests; those are not presented as runtime smoke verification.

The synthetic fixture contains no Pokémon artwork or personal data and remains in ignored `screenshots/`; no screenshot fixture is committed or copied into application storage. Its unchanged SHA-256 is `67F480797C1AB0C6762DEFF4623F20A2A630414D383E3F8BA71DA79198BF918E`. The test app may remain open on the unsaved review; closing it or cancelling releases the preview.

## Review

Existing migrations 1–3 remain unchanged. New commands are enrolled in the Tauri ACL and granted only to the local main window; no filesystem/dialog plugin or broad arbitrary read capability was added. SQL and authoritative validation remain in Rust. No image field exists in save DTOs or the database schema. The native picker reads originals without writing and returns only a sanitized temporary pixel preview. The diff was reviewed for scope, privacy, transaction behavior and numeric precision.

## Follow-up: data-only review and awaiting evaluation

The importer no longer asks for tags and always sends an empty tag list through the unchanged native contract. The record list and save success message explicitly indicate awaiting evaluation. All current records have no completed evaluation; existing tags remain displayed and do not alter that state. No Rust code, migrations, dependency, capability or stored record was changed.

Updated regression tests cover importing without tag controls or assignments, pending presentation after confirmation and remount, retained data on retry, and preservation of existing standard/custom tags. Original Task 003 results above remain historical; the follow-up checks are recorded below.

- `npm run check`: stopped at the same pre-existing Prettier CRLF/LF issues in 20 untouched files; no formatter rules or unrelated files were changed.
- Targeted Prettier checks: all 12 changed source/document files passed.
- `npm run lint` and `npm run typecheck`: passed without warnings.
- `npm test`: 13 tests in 2 files passed outside the sandbox. The initial sandbox attempt again failed before loading tests with a Vitest temporary-file rename `EPERM`; the approved retry passed.
- `npm run rust:fmt` and `npm run rust:check`: passed. Existing incremental-cache hard-link warnings remained; no Rust source changed, so the original 24-test native suite was not rerun for this UI-only follow-up.
- `npm run tauri:build`: passed outside the sandbox using the existing NSIS cache; both the release executable and local installer were rebuilt with the updated UI. The existing `.app` bundle-identifier advisory remained.
- `git diff --check`: passed. Native sources, migrations, dependency manifests/lockfiles and permissions are unchanged.
- Native GUI smoke verification was not repeated; the original partial smoke-test limitations above still apply.
