# Offline screenshot extraction

Selection fills supported visible fields automatically, then requires editable review and explicit confirmation. Native image processing locates appraisal bars and prepares text crops; bundled Tesseract LSTM OCR reads name and CP. Only confirmed structured data is persisted.

## Text recognition

Rust produces metadata-free, thresholded PNG crops from validated full-resolution pixels. Supported inputs are portrait detail screens at least 240 pixels wide, with height between one and three widths. CP is sought in the upper centre (25–75% width, 4.5–10% height). The visible CP prefix must accompany a positive decimal integer; digits alone are insufficient. The name crop is anchored above a unique long central green current-HP line, separate from the appraisal HP IV. Missing, short, red or clipped health lines and unsupported layouts can leave the name unresolved.

CP's small prefix glyphs can be resized geometrically to digit height; no characters are substituted. Tall border-connected phone overlays are removed only when separated from remaining text by a generous blank margin. Text-sized clipped components and overlays touching the final glyph reject the crop.

One classic worker runs Tesseract.js 6.0.1/core 6.1.2 with English/Portuguese language data 1.0.0 (4.0.0_best_int). The installer embeds every resource. `npm run ocr:prepare` copies locked npm assets, checks reviewed versions and produces a SHA-256/size manifest; predev/prebuild run it automatically. There are no runtime downloads or user-installed executable lookups. The worker/core/language paths all use the application origin. Persistent caching is disabled.

The renderer owns the raw worker before initialization, using the inspected pinned worker protocol. Cancel, unmount and the 30-second timeout terminate it immediately; success/failure also terminate it. Upgrades must repeat the protocol/offline test. CSP allows same-origin workers and WebAssembly compilation, without remote sources or broad JavaScript unsafe-eval.

Text admission requires a finite Tesseract score of at least 60/100 and a valid single line. This provisional heuristic is **not calibrated confidence or an accuracy percentage**. Names support letters, marks, numbers, spaces and limited nickname punctuation/gender symbols. Other symbols may require manual entry. No species dictionary, spelling correction or hidden-data inference is used. CP retains exact decimal-string/i64 validation. Raw OCR text is discarded after producing suggestions.

## Appraisal IVs

Rust scans opaque full-resolution pixels before preview resizing. Exactly three aligned tracks must have compatible widths, row groups and near-equal spacing, ordered as attack/defense/HP. Supported evidence includes grey/pale empty tracks, orange partial fills, pink full fills, rounded caps and modest JPEG colour bleeding. Both separator gaps must appear at the thirds; solid rectangles cannot establish a scale. Tracks need at least 60 pixels of width and three agreeing rows. Endpoints must lie within 0.22 of an IV step from an integer; separator endpoints at 5/10 are handled explicitly. Contradictory valid rows reject the group.

All three IVs are emitted together or all remain unresolved. Complete empty/pink tracks can propose 0/15. Cropping, shared overlays, incomplete groups and extra tracks are rejected when identified by evidence gates. Geometry cannot prove game identity or verify text labels; some decoys/occlusions can still match. Rotation, perspective, dark themes, compression, crops and UI changes may need manual entry.

## Contract and review

The existing screenshot response now includes temporary `textRegions` PNG arrays (or null), beside the preview and typed extraction. No IPC command, file capability, SQL access or save field is added. Native text fields initially report unavailable because Rust does not execute OCR; the renderer replaces them before enabling confirmation.

| Property         | Meaning                                                                                                                                                     |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| value            | Name/CP string, IV integer or null.                                                                                                                         |
| state            | detected, unresolved or unavailable.                                                                                                                        |
| confidence       | tentative for an admitted result; none for null. Never an accuracy rate.                                                                                    |
| method           | tesseract_lstm, appraisal_geometry_v1 or unavailable. Geometry's existing identifier denotes rule-based provenance with expanded real-image evidence gates. |
| engineConfidence | Optional raw Tesseract score, not a probability.                                                                                                            |

The preview appears during reading, with an accessible status and Cancel extraction. Fields remain locked until completion. Suggestions are editable with descriptions linked by aria-describedby; manual corrections retain original suggestions. Replacement immediately clears the old draft. Failed saves retain entered values. OCR failure/timeout preserves native IV suggestions and manual fallback. Late results after cancellation/unmount cannot update or save. Explicit confirmation saves reviewed fields with no tags; records remain Awaiting evaluation.

## Evidence and distribution

Synthetic tests cover image/resource boundaries, overlays, IV grouping, resizing/JPEG, ambiguous text, worker failure/cancel/timeout and confirmation. The supplied private screenshot returned Abra, CP 468 and IVs 15/2/2. The real browser worker agreed with external requests blocked and no IndexedDB databases. One known reference is not a held-out benchmark: no general recognition rate is claimed. No private screenshot is committed or bundled.

Optional diagnostics: `node scripts/verify-local-extraction.mjs <authorized-local-image>`. The browser variant additionally requires existing Playwright/browser installations; set TCM_PLAYWRIGHT_MODULE/TCM_BROWSER_EXECUTABLE when needed and optionally TCM_EXPECTED_EXTRACTION (JSON expected values). Crops stay in memory; no record is imported or runtime resource downloaded. Normal tests require no private images.

Existing 10 MiB/4096-side/12-million-pixel/64-MiB decoder limits remain. Native processing adds an RGBA copy and bounded crop/connected-component workspace. WASM adds runtime memory beyond the decoder budget; no hard process-memory or production-latency claim is made. A single worker/timeout bounds concurrency. Assets total about 21.9 MB before notices/compression. Models/generated assets are ignored by Git and included at build time. See [decision 0004](../decisions/0004-offline-hybrid-extraction.md), [licenses](../legal/ocr-notices.txt) and [validation](validation.md).
