# Automatic OCR follow-up validation

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
