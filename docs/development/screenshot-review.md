# Manual screenshot review

## Workflow

In Inventory, choose **Select screenshot** and select one local PNG or JPEG. Read the preview and enter a display name, optional CP, either all three appraisal IVs or none, and any existing standard/custom tags. **Confirm and save** creates a new record. Cancel discards the review. Selecting or cancelling the file picker never writes inventory data. This is manual entry; no OCR or automatic extraction is implemented.

The saved list loads from the native repository on entry to Inventory, including after application restart. A failed save retains review values for correction/retry. The UI locks submission synchronously while saving; after success it uses the returned committed record without a second fallible read. Matching names/CP/IVs create independent UUIDs. Editing, deletion, custom-tag management and filtering remain deferred.

## Native boundary and persistence

Only three application commands are registered and enrolled in Tauri's command ACL via `AppManifest`: `select_screenshot` (no path argument), `inventory_snapshot` (no arguments), and `confirm_inventory_import` (typed reviewed fields/tag IDs). Only the local main window receives `inventory-review`; there are no remote capabilities, filesystem/dialog plugins, arbitrary SQL or arbitrary file readers. Database work and image decoding run on blocking workers; a mutex serializes repository access and a native guard prevents concurrent pickers. Tauri injects the calling window as the native dialog parent; no window or path argument is supplied by the renderer.

The existing Rust repository validates names, CP and IV groups. Its `create_with_tags` method uses one SQLite transaction for a new UUID record and all selected assignments; any validation, assignment or commit failure rolls back. Reads return a consistent snapshot. Existing migrations 1–3 remain unchanged. No new persistence fields are needed.

CP crosses IPC as a decimal **string**, or null for unknown, covering 1–9223372036854775807 (SQLite signed 64-bit maximum). JavaScript never converts CP to Number. Tag identifiers are strings too. IVs are a null group or a typed complete object with each integer in 0–15. Incomplete groups and unknown fields are rejected by deserialization, and domain validation remains authoritative. Storage errors are mapped to safe structured codes/messages, never raw SQL or local paths.

## Image limits and privacy

The native picker selects one existing local file, filtered to PNG/JPEG. Content sniffing, not a filename extension, determines the supported codec. Native validation limits encoded input to **10 MiB**, each decoded side to **4096 pixels**, decoded area to **12,000,000 pixels**, and the decoder output-buffer allocation budget to **64 MiB**. The image library also applies this as a best-effort budget to decoder internals; it is not a hard cap on the process working set. Headers are checked before full decoding; actual decode must succeed. A bounded read also handles files that grow after selection. Only PNG/JPEG decoder features are enabled.

A fresh PNG preview is encoded from decoded pixels, stripping source metadata. The preview is reduced to at most 1600 pixels per side. Only preview bytes and original decoded dimensions cross IPC. The source path and filename stay in the picker/read operation and are never returned, logged or persisted. Original files are opened read-only; the application neither copies them into app storage nor modifies/deletes them. SQLite stores only reviewed structured fields and tag assignments. There are no uploads, network services, analytics or image retention.

The renderer holds a temporary Blob URL, revoking it on replacement, picker cancellation, review cancellation, completion or unmount. Late picker results after unmount are discarded. Temporary Rust buffers and file handles are dropped when selection processing returns. Resource release means dropping references; memory is not cryptographically erased and the OS/native picker may manage its own recent-file history.

## Verification

See the Task 003 entry in `docs/development/validation.md` for exact checks and native smoke-test results. Automated tests cover hostile/unsupported/corrupt images and resource boundaries, complete/unknown/incomplete IVs, exact CP limits, confirmation/cancel, in-flight duplicate prevention, recoverable failures, saved-list rendering, transaction rollback and reopen persistence. No personal screenshots are committed.
