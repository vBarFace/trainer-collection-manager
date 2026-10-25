# Task 003 validation

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
