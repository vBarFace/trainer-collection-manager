# 0003: Bounded experimental local extraction

Date: 2026-10-26. Status: accepted for the Task 004 bounded foundation.

## Decision

Reuse the validated native PNG/JPEG decode and the existing `image` crate for deterministic, conservative appraisal-bar geometry. Return temporary typed suggestions alongside the preview. Keep name and CP explicitly unavailable, with manual input. Do not introduce a database migration, additional IPC command, permission, external executable, model or dataset.

The current implementation demonstrates the rules on original synthetic drawings. No redistribution-authorized labelled Pokémon GO screenshots were supplied, so real screenshot recognition, accuracy and broad layout support are **not demonstrated**. This is a bounded experimental increment, not completion of general screenshot recognition.

## Alternatives assessed

| Approach                                              | Trade-off and disposition                                                                                                                                                                                                                                                                                         |
| ----------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Windows `Windows.Media.Ocr` via Rust Windows bindings | Avoids bundling language models, but Microsoft supports this API in desktop apps with package identity. The current Tauri NSIS distribution has no MSIX identity. Do not depend on unsupported unpackaged behavior or change installer architecture in this task.                                                 |
| Tesseract native engine and language data             | Mature offline OCR, Apache-2.0 engine; requires engine/native packaging plus selected language data, binary provenance, notices and separate data redistribution review. No approved bundle or real screenshot benchmark exists here. Defer rather than depend on a user-installed executable or download models. |
| Rust OCR/model wrappers, ONNX or browser/WASM OCR     | A wrapper alone supplies neither recognition models nor their redistribution rights. Adds model/runtime packaging and memory costs without labelled evidence. Deferred.                                                                                                                                           |
| Deterministic pixel analysis with existing `image`    | Portable, explainable, no incremental dependency or model/license burden. Narrow palette/geometry support and false positives remain possible. Chosen with tentative confidence, strict evidence gates and explicit review.                                                                                       |

Microsoft's [OCR namespace documentation](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr?view=winrt-26100) specifies the package-identity requirement. Tesseract's [installation documentation](https://tesseract-ocr.github.io/tessdoc/Installation.html) describes the engine and separate language data. Sources checked 2026-10-26; no OCR runtime was installed or invoked.

## Consequences

All three IV suggestions are emitted together or all remain unresolved. No hidden data is inferred from species, stars or text. Tentative is a qualitative rule match, never an invented percentage or calibrated probability. Name/CP remain null even for clear visible text. Other graphics can match these rules; users must check every proposed value. See [exact supported evidence](../development/local-extraction.md).

Broader recognition needs authorized labelled reference images, negative/overlay cases, held-out accuracy measurement and review of any new engine/model licenses, resource costs and distribution. An OCR choice or packaging change needs a separate decision. No approval is required to use the implemented manual fallback.
