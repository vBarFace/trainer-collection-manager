# Changelog

All notable changes are documented here.

## [Unreleased]

### Changed

- Inventory now distinguishes Pending, Evaluated, Needs data and Stale appraisal states, with record selection and explicit evaluation/re-evaluation. Import review continues to confirm data without tag selection; existing assignments remain intact.

### Added

- Confirmed saved-IV corrections from the appraisal panel, with prefilled grouped fields, an explicit unknown-group option, cancellation and retained drafts after save failure. A narrow native command updates only IVs, rejects conflicting saved values and uses existing appraisal invalidation; evaluation and tags are never applied automatically. No schema or ruleset change.

- Task 005: migration 4 persists per-Pokémon appraisal metadata and an IV revision counter. The deterministic `appraisal-1` Rust engine reports confirmed IVs, total, descriptive percentage and a perfect-appraisal finding. Typed findings/recommendation foundations are separate from assigned tags; no strategic recommendations or automatic tag assignments are produced. IV changes and ruleset mismatches invalidate previous attempts.

- Automatic offline screenshot name/CP OCR with bundled English/Portuguese Tesseract resources, temporary native crop preparation, real appraisal-bar palettes/rounded-track support, cancellation/timeout and manual fallback. Verified the supplied real example; no general accuracy rate is claimed.

- Task 004: experimental offline appraisal-bar suggestions, typed field-level provenance and confidence, unresolved notices and editable review. Reset drafts on screenshot replacement. Text OCR remains unavailable; only user-confirmed structured data is saved.

- Task 003: bounded local screenshot preview, manual data review, explicit atomic inventory import and persistent Inventory list. OCR remains deferred.

- Initial Tauri 2 desktop foundation, local SQLite migration infrastructure, quality tooling and project governance documents.
