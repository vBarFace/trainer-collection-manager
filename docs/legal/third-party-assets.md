# Third-party assets and data

No third-party game images, sprites, logos or game datasets are included. Generic OCR language data is obtained through locked npm packages and bundled at build time; model bytes and generated assets are not committed.

The MIT license covers only original project code and documentation. Pokémon, Pokémon GO and related marks belong to their respective owners and are not licensed by this project. Before adding any external asset, dataset, library content or model, record its source, license, redistribution rights, attribution and update/removal plan.

## Task 003 native dependencies

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
