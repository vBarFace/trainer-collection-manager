# Development roadmap

1. Implemented (Task 002): local inventory schema, stable UUIDs, validation, grouped optional appraisal IVs, standard/custom tags and typed Rust persistence.
2. Implemented (Task 003): native PNG/JPEG selection, bounded temporary preview, manual review, explicit atomic confirmed saving and a minimal persistent **Awaiting evaluation** Inventory list. Import review confirms data without selecting classification tags; previously saved tags remain intact. OCR and automatic extraction are deferred.
3. Implemented bounded increment (Task 004): typed per-field extraction, conservative experimental appraisal-bar geometry, editable tentative suggestions and safe manual fallback. No new dependencies or persistence fields. Name/CP OCR and real-image accuracy validation remain unsupported; see [local extraction](local-extraction.md).
4. Future: obtain redistribution-authorized labelled screenshots and validate/tune bar detection; separately review a distributable offline OCR engine, then a persisted evaluation lifecycle, versioned deterministic analysis, classification tags and explainable recommendations; only then consider optional local AI proposals behind typed preview tools. Full editing, deletion, filtering and custom-tag management UI remain deferred.

Each milestone requires separate product, security and license review.
