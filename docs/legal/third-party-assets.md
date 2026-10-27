# Third-party assets and data

No third-party game images, sprites, logos or game datasets are included. Generic OCR language data is obtained through locked npm packages and bundled at build time; model bytes and generated assets are not committed.

The MIT license covers only original project code and documentation. Pokémon, Pokémon GO and related marks belong to their respective owners and are not licensed by this project. Before adding any external asset, dataset, library content or model, record its source, license, redistribution rights, attribution and update/removal plan.

## Proposed strategic data and training review (Task 008)

No new data, model, dependency or third-party material is included by this documentation task. Task 007's dataset-specific rights gate remains unresolved; this section defines future evidence requirements, not a new legal assessment or permission to collect data. Preserve existing source research and adapter provenance as the reference-data source of truth.

Evaluate each intended use separately for each exact source, field subset and revision:

| Intended use                  | Evidence to record before that use                                                                                                                                                                                     |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Collection/access             | Authorized API terms or applicable scraping/TDM basis, access restrictions, source authority, permitted scope and collection method. Public access alone is insufficient evidence.                                     |
| Retention/internal processing | Allowed stored fields, duration, purpose, attribution, privacy/consent limits and removal/update obligations, including derived labels.                                                                                |
| Model training/evaluation     | Permission or documented applicable basis for that purpose, derivative-feature/label constraints, contribution consent and applicable opt-outs. Access/retention approval does not automatically approve training.     |
| Dataset redistribution        | Authority over the exact included facts/labels/derived records, downstream license, redistribution/derivative permissions and required notices; software licensing alone does not establish underlying dataset rights. |
| Model redistribution          | Separate review of training provenance and applicable restrictions on weights/features/outputs, model license/notices and update/removal plan. Do not assume weights are unrestricted because raw data is omitted.     |

Potential inputs include legitimate APIs, permitted scraping/TDM, original structured observations and consented contributions. None automatically establishes recommendation-label quality. Keep source URL, revision, retrieval date, exact-byte digests, transformations, label rubric and rights evidence with future dataset/model cards. Evaluate uncertain uses before proceeding; fictional fixtures can validate contracts while real-data use remains blocked.

Inventory saving, local recommendation use and training contribution are distinct choices. Default screenshot retention remains off. Future training contributions require a separate explicit opt-in explaining the exact structured fields, purpose, destination, retention, allowed redistribution and withdrawal/removal limits, especially for already redistributed artifacts. Prefer minimal de-identified structured contributions; do not include screenshot bytes, paths, account identifiers or personal notes by default. No collection/upload capability is authorized or added here.

## Task 006 reference foundation

No third-party GO dataset or artwork is included. [Source research](../development/reference-data-sources.md) records candidate upstreams, licensing evidence and limitations. The production manifest is original and empty; fictional fixture records are original MIT test data, not assertions about real Pokémon. `serde_json` moves from development-only to a direct runtime dependency for snapshot parsing. The already resolved crate declares MIT OR Apache-2.0; no package or game asset is newly downloaded. Future data inclusion requires a separate dataset-specific rights/provenance review and retained notices.

## Task 003 native dependencies (historical)

No third-party screenshot or game asset is included. New direct native dependencies are:

| Crate                           | Version requested | License           | Reason                                                                                                                   |
| ------------------------------- | ----------------- | ----------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `image`                         | 0.25.10           | MIT OR Apache-2.0 | Content detection, bounded PNG/JPEG decode and pixel-only PNG preview. Default features disabled; only PNG/JPEG enabled. |
| `rfd`                           | 0.17.2            | MIT               | Native single-file picker called only in Rust; no renderer filesystem or generic dialog API.                             |
| `serde`                         | 1.0.229           | MIT OR Apache-2.0 | Typed native input/output serialization; already transitive through Tauri.                                               |
| `serde_json` (development only) | 1.0               | MIT OR Apache-2.0 | Tests exercise actual IPC JSON shape and numeric boundaries; already transitive through Tauri.                           |

Licenses and required codec/picker features were reviewed against crate package manifests and source in the Cargo registry. Cargo.lock records exact resolved versions and checksums, including transitive dependencies. Upgrades must retain the restricted codec feature set and repeat limit/metadata regression checks. Remove these direct dependencies if screenshot review/typed IPC is removed.

## Task 004 initial bounded extraction (historical)

No new dependencies, executables, models, datasets, fonts or game assets. Extraction uses the existing MIT OR Apache-2.0 `image` crate and original MIT Rust/TypeScript code. Test fixtures are original geometric drawings generated in memory; no user screenshots or third-party reference images are committed.

Tesseract (Apache-2.0 engine plus separately selected language data) was assessed but not bundled; no unreviewed binaries/traineddata are downloaded. Windows OCR was assessed but not added because the current installer does not provide its required package identity. See [technical decision](../decisions/0003-bounded-local-extraction.md) for source references and future licensing/distribution review requirements.

## Task 004 automatic OCR follow-up

Tesseract.js 6.0.1 (Apache-2.0), core 6.1.2 (Apache-2.0 plus bundled component notices) and English/Portuguese language packages 1.0.0 are reviewed and locked. Language package metadata declares MIT for packaging; upstream trained data is Apache-2.0. Existing Rust image primitives handle bar detection and crop segmentation; no OpenCV native dependency is added.

`npm run ocr:prepare` checks exact reviewed versions, copies resources and complete licenses into ignored `public/ocr`, and records generated-file SHA-256/size metadata. Vite/Tauri embeds them in the installer. No private image, game dataset, character artwork or model download at runtime is introduced. Full notices/provenance are in [ocr-notices.txt](ocr-notices.txt) and [ocr-third-party-licenses.txt](ocr-third-party-licenses.txt), including statically linked codecs/Leptonica and the worker's unmodified extracted notice file. Package upgrades require repeating this review.

Primary API references: [image 0.25.10 limits](https://docs.rs/image/0.25.10/image/struct.Limits.html), [ImageReader](https://docs.rs/image/0.25.10/image/struct.ImageReader.html), [rfd 0.17.2 native file dialog](https://docs.rs/rfd/0.17.2/rfd/struct.FileDialog.html). Width/height checks and the output-buffer reservation are enforced; decoder internal allocations use a best-effort budget.

Newly resolved transitive packages were also reviewed: `byteorder-lite` 0.1.0 (Unlicense OR MIT), `bytemuck` 1.25.2 (Zlib OR Apache-2.0 OR MIT), `moxcms` 0.8.1 and `pxfm` 0.1.30 (BSD-3-Clause OR Apache-2.0), and `zune-core` 0.5.3 / `zune-jpeg` 0.5.15 (MIT OR Apache-2.0 OR Zlib). Their declared licenses permit use and redistribution with the applicable attribution/license notices.
`pollster` 0.4.0 (a platform-specific `rfd` dependency, not compiled in this Windows build) declares Apache-2.0/MIT; reviewed from its [published manifest](https://docs.rs/crate/pollster/0.4.0/source/Cargo.toml).
