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
stack on the prior unmerged increment. The brief requests focused PR delivery;
repository AGENTS.md still requires explicit approval at remote publication.
No pull-request merge or release publication is performed. Original code remains MIT;
upstream data rights remain separate.

## Published review stack

The seven branches were pushed with explicit authorization. Review in dependency
order: each PR targets the preceding feature branch, with PR 0 targeting main.
PR 0 is ready for review; PRs 1–6 are drafts because the broader gameplay and
real-image acceptance gaps remain open. Local feature-branch merges propagate
fixes without rewriting history; no GitHub PR has been merged.

| Increment | Pull request                                                          | User-visible result                                                                                  |
| --------- | --------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| PR 0      | [#14](https://github.com/vBarFace/trainer-collection-manager/pull/14) | Await observable history invalidation and asynchronous focus restoration; Windows LF checkout policy |
| PR 1      | [#15](https://github.com/vBarFace/trainer-collection-manager/pull/15) | User-initiated offline reference import, provenance, activation and rollback                         |
| PR 2      | [#16](https://github.com/vBarFace/trainer-collection-manager/pull/16) | Batch review/grouping, searchable inventory, manual optional observations, protection and tags       |
| PR 3      | [#17](https://github.com/vBarFace/trainer-collection-manager/pull/17) | Collection comparisons and explicitly bounded theoretical league statistics                          |
| PR 4      | [#18](https://github.com/vBarFace/trainer-collection-manager/pull/18) | Explainable recommendations, owned alternatives, global goals and budget evidence                    |
| PR 5      | [#19](https://github.com/vBarFace/trainer-collection-manager/pull/19) | UUID-linked review/compare/wait roadmaps with progress and stale-context detection                   |
| PR 6      | [#20](https://github.com/vBarFace/trainer-collection-manager/pull/20) | Private structured export, real-source validation tooling and a local Windows preview installer      |

The combined frontend checks after the synchronization follow-up passed:
formatting, lint, strict TypeScript, 144 UI tests and 8 adapter tests. The focus
fixes await React effects without timers, relaxed assertions or production UI
changes. Each PR links its current frontend and native CI checks.

The next acceptance work is representative authorized screenshot extraction and
an installed-app offline journey, followed by the still-unverified source-field
and gameplay mechanics in the [alpha acceptance matrix](mvp-alpha-validation.md).
This stack delivers the collection/statistical preview; it does not establish
full gameplay-MVP or personal-alpha acceptance.

## Executed baseline

Final reference-validation follow-up: pinned real PvPoke and GM files plus three
PoGoAPI endpoints passed the offline developer validator. Nine independent
hypothetical statistical cases passed; three source base-stat samples agreed.
This supersedes the initial structural-compatibility uncertainty below, without
claiming full mechanics or data-rights acceptance. [Reproducible evidence](mvp-real-reference-validation.md).

PR 6: screenshot/catalog-free structured export, individual source-artifact
hashes and multi-file PoGoAPI import, 100-image/1,000-record checks, and a
history-aware cross-instance optional-observation confirmation fix. Final checks:
144 frontend, 8 adapter, 160 native, 2 native reference CLI tests passed; 2
authorized-sample tests ignored. Rust format/check, Windows release/NSIS build,
and native app launch/navigation passed. See [actual acceptance and artifact](mvp-alpha-validation.md).
Full gameplay MVP acceptance remains open for the explicitly listed data,
mechanics, real-image and installed-app verification gaps.

PR 5: per-UUID roadmap UI, required/optional steps, postpone/ignore/reset,
explicit review-confirmation handoff, stale controls and previous-plan context.
The six recommendation/roadmap UI tests passed. See [roadmap behavior and limits](mvp-roadmaps.md).

PR 4: functional Recommendations navigation and native collection service,
supporting facts/uncertainty, owned alternatives, versioned league statistics,
retention overrides and explicit global goals/budget. Six integrated service
tests passed; recommendation/navigation/inventory UI subset: 46 passed.
The bounded roadmap persistence backend is included with the service; its
progress UI is the next increment. See ADR 0009.

PR 3: pure `collection-1` Rust statistical engine and collection-aware safe
guidance. Eleven integrated engine tests passed, including hand-calculated
numbers, CP caps, level ceilings, protected retention, contradictory/missing
observations and exceptional mechanics. [Scope and limits](mvp-strategy.md).

PR 2: batch review, searchable/filterable paged inventory, typed optional manual
observations, explicit protection/favorite distinction, and applied tag editing.
See [behavior, persistence and remaining OCR acceptance](mvp-import-and-inventory.md).
Before final batch enhancements: 132 frontend tests and 135 native tests passed,
with 2 sample-dependent native tests ignored. Final counts are recorded below
after the integrated validation run.

PR 0: local commit `b29fdf9`, branch `fix/power-up-history-synchronization`.
Frontend checks passed (122 tests and 8 adapter tests); native format/check and
122 native tests passed, 2 authorized-sample tests ignored. These are the initial
pre-publication results; the review stack above records subsequent publication.

PR 1: `feat/local-reference-installation` adds bounded offline provider adapters,
Settings, per-user transactional activation/rollback and persistent identity and
version continuity. Four adapter tests, two activation tests, and two Settings
tests passed. [Coverage and unverified fields](mvp-reference-coverage.md) are
explicit. No current real Game Master compatibility or dataset redistribution
rights claim is made. No downloader or background networking is introduced.

- Windows isolated Power Up suite: 21 passed.
- Windows full frontend baseline: 122 passed (the CI timing failure did not
  reproduce on this run).
- Baseline `npm run check` stopped on LF/CRLF formatting differences in the
  existing checkout; `.gitattributes` now prescribes LF text checkout.
- GitHub CI log inspection confirms the exact failed assertion at
  `PowerUpPanel.test.tsx:408`; native CI passed on the same commit.
