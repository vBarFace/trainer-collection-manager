# 0004: Hybrid offline OCR and native appraisal processing

Date: 2026-10-26. Implements the user's automatic name/CP/IV follow-up, superseding decision 0003's OCR deferral. The previous bounded milestone remains historical.

Use Rust image processing for appraisal bars and bounded text crops, plus bundled Tesseract LSTM in a WebAssembly worker for name/CP. File selection, validation, SQL and persistence stay native. Recognition produces editable suggestions, never save actions.

## Alternatives and trade-offs

- Native Tesseract libraries/executables require additional Windows binary packaging. WASM uses locked npm packages and existing Vite/Tauri assets, adding about 22 MB unpacked and worker memory. The actual worker is tested with the shipping CSP.
- Windows OCR still requires package identity; changing NSIS is unnecessary.
- OpenCV was considered for segmentation/contours. Existing Rust image primitives supply pixel access, cropping, connected components and resizing without another native runtime.
- A learned detector/general vision model would add costs without demonstrated need for these structured regions. Revisit broader layout support after a representative labelled benchmark.

The user authorized implementation after discussing these trade-offs. New packages are Tesseract.js 6.0.1 and English/Portuguese language data 1.0.0; the lockfile resolves core 6.1.2. Full component licenses accompany generated resources. Model bytes stay outside Git. Runtime selects local resource paths only, and preparation rejects unreviewed version changes.

## Boundaries and evidence

The existing screenshot response adds metadata-free PNG crops; no additional command, filesystem/shell capability or save field is added. A renderer worker produces typed results and uncalibrated engine scores, then terminates. Its pinned protocol adapter owns the worker from creation, enabling cancellation during initialization. Package upgrades must repeat protocol, CSP, timeout and offline tests. No cloud service, account access, species dataset, local conversational model or game automation is introduced.

Local recognition on the supplied screenshot returned Abra, CP 468 and IVs 15/2/2. A Chrome worker test with Tauri's CSP, remote requests blocked and no IndexedDB databases agreed. One example is not an accuracy study. Hidden/unreadable/ambiguous fields remain unresolved; confirmed IVs still form a complete group or null. See [limitations](../development/local-extraction.md) and [validation](../development/validation.md).

Primary references: [local resource configuration](https://github.com/naptha/tesseract.js/blob/v6.0.1/docs/local-installation.md), [worker implementation](https://github.com/naptha/tesseract.js/blob/v6.0.1/src/worker-script/index.js), [language data](https://github.com/naptha/tessdata), [Windows OCR restriction](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr?view=winrt-26100). Pinned component/license provenance appears in `docs/legal/ocr-third-party-licenses.txt`.
