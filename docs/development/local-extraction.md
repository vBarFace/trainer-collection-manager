# Experimental local extraction

## Supported evidence

Selection runs a pure Rust scan over the validated full-resolution RGBA pixels before preview resizing. It searches across the image rather than assuming a phone resolution or fixed screen coordinates. A match requires exactly three tracks ordered top to bottom (attack, defense, HP), with equal starts, widths and thicknesses, near-equal spacing and visible margins. Each track needs at least 60 pixels of width, three consistent rows, an aspect ratio of at least 12, and two visible separator gaps at its thirds. Fill must progress left to right without restarting, using the narrow orange/pale-track palette, or the distinct pink full-bar palette. Endpoints must quantize to 0–15 within at most roughly one pixel, bounded to 0.15 of a step.

The two separator gaps provide evidence of full track length: a shared overlay shortening all tracks must not redefine the IV scale. Continuous rectangles without separators are unsupported. A complete empty track can propose 0; a complete pink segmented track can propose 15. Orange spans without remaining track evidence are rejected. Unexpected fourth tracks, inconsistent rows, missing tracks, insufficient resolution, clipping, occlusion and unsupported colors/layouts resolve to no IV group. Pixels with alpha below 250 do not count as colored evidence. Cropping outside the complete tracks can work; cropping through them is unsupported. Rotation, perspective, dark themes, rounded/antialiased edges, lossy compression and unusual overlays may prevent detection. Some visually plausible decoys or overlays can still match; geometry does not identify the game or verify labels.

This palette and geometry have **synthetic coverage only**. No real screenshot reference corpus was available, no recognition rate is claimed, and most unsupported variants should be entered manually. These rules deliberately do not claim arbitrary scale/crop/quality robustness. Name and CP text recognition are unsupported regardless of text clarity; hidden CP stays null. No OCR parser or text-recognition accuracy tests are represented as working functionality.

## Contract and review

`ScreenshotPreview.extraction` contains `displayName`, `cp`, `attack`, `defense`, `hp` and a list of recoverable notices (`code`, safe `message`). Each field has:

| Property     | Meaning                                                                                                                                                             |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `value`      | String for name/CP, integer for IV, or null. CP retains the existing decimal-string contract.                                                                       |
| `state`      | `detected`, `unresolved` or `unavailable`. Unresolved means the attempted rule had insufficient/ambiguous evidence. Unavailable means no engine supports the field. |
| `confidence` | `tentative` for a bar rule match; `none` for null. No numeric scores or accuracy probabilities.                                                                     |
| `method`     | `appraisal_geometry_v1` for IV attempts; `unavailable` for text.                                                                                                    |

`text_unavailable` and `bars_unresolved` notices are recoverable: the preview and manual form remain usable. Extraction cannot write inventory. Failed image validation still rejects the image before extraction. All IV suggestions are present together or absent together; the unchanged frontend/native validation also requires all three confirmed IVs or none.

Suggested values prefill the editable review. Each input is associated with its provenance through `aria-describedby`; editing changes the description to manual entry/correction and retains the original suggestion for comparison. Replacing an image resets both draft values and provenance. Save failure retains them. Cancel, completion, replacement and unmount release the preview and discard suggestions. Only **Confirm and save** submits the existing structured input, with no tags, image or extraction metadata. Records remain **Awaiting evaluation**.

## Resource, privacy and distribution boundaries

The existing input/decode/preview limits and native picker ACL remain unchanged. The scan is linear in pixel count with a row buffer and at most 128 candidate bands; excessive candidates yield unresolved. Decoding/conversion and preview encoding retain their existing allocations; RGBA analysis temporarily adds up to four bytes per decoded pixel. This is not a hard cap on process working memory. There are no threads per pixel, background retries, model downloads or network calls. Analysis runs on the existing blocking worker; review appears when selection processing finishes. No source file is changed or retained.

No new dependency or redistribution asset is added. See [decision 0003](../decisions/0003-bounded-local-extraction.md), [licensing](../legal/third-party-assets.md), [screenshot review](screenshot-review.md) and [validation](validation.md). Tests generate original MIT geometric images in memory. Synthetic test counts and timings measure implementation behavior only; real-image accuracy requires an authorized labelled corpus with negative examples and held-out evaluation.
