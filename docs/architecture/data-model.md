# Data model

## Implemented schema

`schema_migrations` records applied immutable migration versions. `application_settings` stores local application metadata by key. No player or Pokémon data exists yet.

## Planned inventory invariants

- Each individual Pokémon receives an immutable stable UUID, independent of display name or order.
- Structured fields, source provenance, validation state and timestamps are explicit.
- Imported screenshot data must be user-reviewed before persistence; original image files are not retained by default.
- Tags use a constrained representation plus custom user tags; appraisal/second attack are data fields rather than implicit tags.
- Formula and game-data versions are recorded alongside derived recommendations.

The full schema remains pending requirements and license review for any reference data.
