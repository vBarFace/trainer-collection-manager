# Deterministic MVP preview validation

Date: 2026-11-01. Base: remote/local main `4378a15`. Delivery uses separate stacked
feature branches; no main commit, PR merge or release publication is authorized
by this record. The full gameplay MVP is **not accepted** by this preview.

## Implemented vertical slice

Local reference installation → bounded screenshot queue/review → confirmed UUID
inventory → safe collection/statistical guidance → individual review/compare/wait
roadmap → explicit confirmed action → stale detection/recalculation → restart →
structured export. Existing S2 lifecycle actions and utility-1 contracts are reused.

## Executed checks so far

- PR 0 full frontend baseline: 122 passed; reference adapter: 8 passed;
  native: 122 passed, 2 sample-dependent ignored. Remote main's frontend failure
  was verified from its actual CI log and corrected as an asynchronous assertion.
- Batch/inventory integration: frontend 135 passed, adapter 8 passed;
  native 136 passed, 2 ignored.
- Pure strategy module: 11 integrated tests passed, including independently
  hand-calculated fictional CP/HP/stat-product examples and overflow/cap limits.
- Strategy service: 6 integrated tests passed for protection, preferences,
  context/revision checks, bounded plans, restart and no-CP Power Up staleness.
- Recommendation/roadmap UI subset: 6 passed.
- Structured backup: 1 integrated test passed outside the Windows sandbox after
  the sandbox denied atomic file rename. Raw identity pins survive provider
  switches; unused custom tag definitions are retained; catalog payloads excluded.
- Fictional native end-to-end offline workflow: 1 passed through source
  normalization, import retries, identity, duplicate analysis, preference/progress,
  confirmed Power Up, recalculation, restart and export. This is not a real-image
  extraction or real-source compatibility claim.
- Final `npm run check`: formatting, lint, strict TypeScript, **144 frontend
  tests**, and **8 adapter tests passed**.
- Final Rust format/check passed; **160 native tests passed**, 2 authorized-sample
  tests ignored. `reference:test-native`: **2 passed** against the production CLI.
- Volume checks: 100 generated PNG files decoded sequentially without changing
  originals; 100 mocked OCR jobs remained sequential with no unconfirmed import;
  1,000-record inventory search/page rendering passed. These are not real-image
  OCR accuracy or benchmark timings.
- Cross-instance regression: optional-details drafts carry a lifecycle evidence
  token, read in one SQLite snapshot. Another instance's history-only Power Up
  rejects an older draft even when strategic observation revision/CP remain
  unchanged. The final full suites above include this test.
- Final `npm run tauri:build` passed: Windows x64 release executable and NSIS
  installer built. Packaging required the NSIS tool download; the smoke-test
  executable was closed before the successful rebuild.
- Native Windows launch/navigation smoke passed on an empty collection:
  Inventory, Recommendations and Settings loaded with successful native commands.
  No observations, preferences or reference data were changed in the GUI.

## Local build artifact

Ignored output: `src-tauri/target/release/bundle/nsis/Trainer Collection Manager_0.1.0_x64-setup.exe`
(11.67 MiB). SHA-256:

```text
9C228AB9A05C20D80896895F88626DCB5C4CD2F7317BC12EF36250CDDCEBA9AF
```

This is a local preview artifact, not a published release or full-MVP acceptance.
The installer wizard was not executed. No disconnected installed-app test or
representative real-data screenshot acceptance was performed.

## Remaining release acceptance

The latest user-confirmed scope limits OCR to name, CP and appraisal IVs; all
other fields stay manually entered and optional. Prioritize gym attack/defense,
Team Rocket and transfer-candidate review before expanded PvP. This changes the
acceptance scope, not the implemented mechanics or previously executed results.
See the [current priority contract](mvp-current-priorities.md).

Subsequent source validation: current pinned PvPoke/GM inputs and separately
hashed PoGoAPI responses passed the production adapters. Nine hypothetical
real-reference statistical cases matched independent exact rational arithmetic;
three sampled base-stat records agreed across all providers. See [revisions,
fingerprints, counts and limitations](mvp-real-reference-validation.md). No real
dataset records or derived sample fixture records are committed.

| Acceptance area                              | State / required evidence                                                                                        |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Stable UUIDs, corrections, lifecycle history | Existing and new native integration tests; no game automation                                                    |
| Local reference install/rollback/restart     | Fictional persistence tests and current pinned real-source structural validation passed                          |
| Real provider field coverage                 | Bounded ordinary forms/stats/types/CPM only; provider snapshots remain alternatives, not merged fields           |
| Bulk review/grouping/idempotent save         | Fictional frontend/native tests pass; real representative throughput/accuracy unmeasured                         |
| Extended visible screenshot fields           | Manual and optional by approved scope; automatic extraction deferred, not required for current MVP               |
| Search/filter/duplicates                     | Implemented; exact CP string comparisons and unknown identity tested                                             |
| Statistical league scope                     | Modeled ceiling shown; nine independent hypothetical real-reference checks passed; broader mechanics open        |
| PvE/gym/Rocket/meta/rarity and investments   | Unsupported; missing evidence blocks spending/disposal rather than generating false scores                       |
| Roadmaps/progress                            | Bounded collection/statistical review/compare/wait plans implemented; executable investment plans gated          |
| Structured export                            | Implemented/tested; no backup import or catalog redistribution                                                   |
| Windows installer / installation / live UI   | NSIS build and native launch/navigation passed; installer wizard and real-data UI journey unverified             |
| Offline desktop execution                    | No application networking; native fictional flow executed locally; disconnected installed-app smoke remains open |
| Dataset rights                               | Software licenses/public access do not establish underlying game-data rights; unresolved                         |

No real source dataset, screenshot, private collection, secret, asset or model
weight is committed. The original source remains MIT. See the field-level
[coverage and rights matrix](mvp-reference-coverage.md) for source-specific limits.
