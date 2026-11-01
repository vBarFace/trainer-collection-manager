# Intelligent MVP delivery audit

Audited 2026-11-01 against remote/local main `4378a15`. No open PRs were
returned by GitHub. Main CI run 40602113292 passed native checks and failed
one frontend assertion (Power Up history invalidation).

| Area            | Existing implementation                                                                                                   | Concrete gap / delivery                                                                                                 |
| --------------- | ------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Lifecycle       | SQLite migrations 1–8, UUIDs, CP/IV/identity corrections, confirmed evolution, Power Up, targeted screenshot observations | Reuse; fix asynchronous history assertion first (PR 0)                                                                  |
| Reference       | Schema 1 validation, identity pins, Task 007 local PvPoke adapter and identity ledger                                     | User-local three-source acquisition, normalized mechanics, durable activation/rollback, coverage/rights manifest (PR 1) |
| Import          | Single-image native picker, local Tesseract name/CP, conservative IV geometry, editable confirmation                      | Bounded multi-file review queue, optional structured evidence, explicit grouping, resource cleanup (PR 2)               |
| Inventory       | Persistent list, standard/custom repository tags, detail corrections                                                      | Search/filter/duplicates, individual properties and preferences, applied-tag controls (PR 2)                            |
| Evaluation      | Appraisal only; isolated utility-1 contracts and nine fictional scenarios                                                 | Rust contextual collection comparisons and verified league calculations; abstain for unsupported combat contexts (PR 3) |
| Recommendations | Placeholder                                                                                                               | Prioritized facts, uncertainty, alternatives, versions and prerequisites (PR 4)                                         |
| Roadmaps        | ADR/utility contracts, confirmed action foundation                                                                        | Bounded UUID-linked progress, confirmation handoffs, stale-context checks (PR 5)                                        |
| Alpha           | Frontend/native CI and Tauri build                                                                                        | Structured backup, full offline/restart flow, real screenshot benchmark, Windows installer validation (PR 6)            |

## Dependency order and acceptance

0. Stabilize CI independently. The remote failure asserts immediately after a
   different component's success message; HistoryPanel invalidates in a React
   effect on the new record object, even when CP/revision do not change. Await
   that observable invalidation rather than a timer. No native/UI behavior change
   is justified by this failure. Normalize Git text checkout to LF for Windows.
1. Implement user-initiated local reference activation using the existing
   Catalog validator/pins. Never bundle or commit upstream game datasets. Record
   source revisions, dates, digests, rights notices and unsupported fields.
2. Add bounded batch review and structured optional facts without changing
   ordinary-import identity semantics. Add inventory search/filter/duplicates.
3. Connect a pure deterministic Rust engine to confirmed observations and real
   local reference facts. Validate independently calculated numerical examples;
   missing mechanics produce limited or unsupported results.
4. Surface safe collection-aware recommendations with per-role explanations,
   then persist small individual roadmaps/progress and reuse confirmed actions.
5. Verify migrations, restart, failed/corrupt imports, protection, stale progress,
   offline behavior and build the Windows alpha. A real OCR accuracy benchmark
   requires representative authorized samples, which are not in this checkout.

Each increment uses a dedicated local branch and conventional commit. Branches
may stack on the prior unmerged increment. Remote push/PR creation and release
publication require explicit authorization under repository AGENTS.md. No merge
is performed. Original code remains MIT; upstream data rights remain separate.

## Executed baseline

- Windows isolated Power Up suite: 21 passed.
- Windows full frontend baseline: 122 passed (the CI timing failure did not
  reproduce on this run).
- Baseline `npm run check` stopped on LF/CRLF formatting differences in the
  existing checkout; `.gitattributes` now prescribes LF text checkout.
- GitHub CI log inspection confirms the exact failed assertion at
  `PowerUpPanel.test.tsx:408`; native CI passed on the same commit.
