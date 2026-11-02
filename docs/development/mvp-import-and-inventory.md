# Batch review and collection management

This increment reuses native image limits, offline Tesseract name/CP extraction,
conservative appraisal-bar detection, grouped IV validation and confirmed import.
It does not introduce a second inventory or change ordinary import UUID rules.

The native picker accepts up to 500 PNG/JPEG paths, retained in memory only.
The renderer receives an opaque temporary session and count. Each image is
decoded separately through the existing bounded reader. Original screenshots
remain untouched. Queue cancellation/unmount releases sessions, workers and
object URLs; saved individuals remain saved. File names, paths and image bytes
never enter the inventory or structured backup.

Queue items distinguish queued, needs review, missing optional information,
ready to confirm, extraction failed, saved and skipped. Extraction remains
tentative; every saved individual requires explicit review. Correcting a draft
does not restart the batch or automatically merge similar images. Updating an
existing UUID continues to require the existing selected-individual workflow.
An explicit sequential extraction pass processes queued files with one image
and worker at a time. Users can explicitly group unsaved screenshots; conflicting
values become unknown and the combined draft requires a new review confirmation.
Each batch draft retains a request UUID. Native atomic request-to-individual
mapping makes identical retries idempotent across restart and rejects altered
facts on an already confirmed request, including after a lost response.

Inventory search covers nicknames and confirmed species/forms. Filters cover
exact decimal-string CP bounds, total appraisal IVs, applied tags/roles,
appraisal status, known individual properties and resolved same-form duplicates.
Unknown identities never count as duplicates. Rendering starts with 100 matching
individuals and expands in batches of 100 rather than rendering thousands of
detail controls at once. No performance timing claim is made.

Optional manual observations include HP, weight/height with units, explicit
gender, catch date, moves, level with evidence, Shiny/Shadow/Purified, favorite,
and separate irreversible-action protection. Unknowns remain null. No appearance
rule infers Shiny status. Source is explicitly manually confirmed; these extra
fields are **not** automatically recognized in this increment. Native validation
rejects contradictory Shadow/Purified, invalid dates/units, impossible HP groups
and unsupported levels. Unknown moves are not assumed legal move choices.

Migration 10 stores typed details, evidence revision and an invalidation flag.
Actual corrections and reconfirmation advance the independent strategic
revision; no-op saves preserve it. Existing corrections make older details need
reconfirmation. A confirmed evolution or Power Up also invalidates optional
evidence, including a Power Up with no CP patch: no new level/HP is fabricated.
UUIDs, existing migrations and appraisal invalidation remain intact.

Applied standard/custom tags are explicitly editable and separate from engine
suggestions. Tag writes atomically compare expected assignments. No suggestion
applies tags or changes favorites automatically. SAVE/SPECIAL and the separate
protection control constrain later assessments.

## Remaining extraction acceptance

No representative authorized screenshot set is present in this checkout.
Therefore no new OCR accuracy percentage, variant detection performance,
ordinary-layout coverage claim or hundred-image extraction timing is established.
The user-confirmed MVP extraction scope is name, CP and appraisal IVs only.
HP, dimensions, dates, moves, symbols and variants remain manual and optional;
their automatic extraction is deferred. This is an accepted scope boundary,
not a missing requirement for the current screenshot workflow. Representative
accuracy validation is still needed for the supported name/CP/IV fields. See
the [current delivery priorities](mvp-current-priorities.md).
