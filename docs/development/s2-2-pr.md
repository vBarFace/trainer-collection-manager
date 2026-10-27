# PR title

feat: confirm evolution and re-observe existing Pokémon

## Summary

A saved Pokémon can now record a user-confirmed normal evolution and optionally update CP/IV observations from a targeted screenshot while keeping its UUID. Evolution validates catalog targets, clears obsolete CP and preserves IVs, tags, nickname and appraisal. Screenshot updates require explicit field and identity review; ordinary import still creates new individuals.

Migration 7 adds confirmed before/after history. Immediate transactions, observation revision checks, catalog version checks and unique request IDs prevent partial writes, stale updates and repeated confirmation. A guarded revision increment covers species-pin changes whose text labels remain identical. Recovery after lost replies reloads authoritative data. See [workflow](lifecycle.md) and [validation](validation.md).

## Verification

- [ ] `npm run check`: blocked at 63 pre-existing CRLF/LF formatting differences in untouched files. Read-only normalization audit passes; changed files pass targeted formatting. Remaining stages passed separately.
- [x] `npm run lint`, `npm run typecheck`
- [x] `npm test`: 100 passed / 8 suites
- [x] `cargo test --manifest-path src-tauri/Cargo.toml`: 113 passed / 2 existing private-image diagnostics ignored
- [x] `npm run rust:fmt`, `npm run rust:check`
- [x] `npm run reference:test`: 8 passed; `npm run reference:test-native`: 2 passed
- [x] `npm run tauri:build`: Windows executable and 11.50 MiB x64 installer built; not installed
- [x] Documentation and migration tests updated; `git diff --check` passes

Vitest/reference tests required approved execution outside the sandbox after temporary-file rename `EPERM`; packaging required it after NSIS download host resolution failed. No native GUI smoke test is claimed.

## Risks and limitations

Production reference data remains empty pending dataset rights, so real evolution targets are unavailable. Test-only hand-authored relationships validate the mechanics without adding a game dataset. Normal permanent evolution semantics remain a reviewed catalog/adapter contract; descriptive requirements are not executed as eligibility rules. Moves, level and observed HP do not exist in this schema and are not fabricated. Screenshot identity requires user verification. Confirmed history covers these workflows only and cascades with existing individual deletion.

Migration 7 is additive; versions 1–6 are unchanged. New commands use the existing main-window permission. No dependency, broad capability or strategic recommendation is added. S2.3 should introduce one explicit selected goal/context per individual.

## Privacy and licensing

- [x] No user data, screenshots, secrets, model weights or generated build output included
- [x] No external dataset or new dependency added; existing production catalog unchanged
- [x] Original screenshots opened read-only; temporary preview/OCR state released

Prepared locally on `feat/s2-2-evolution-reobservation`. No push or remote PR has been authorized or performed.
