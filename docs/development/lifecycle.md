# Confirmed evolution and targeted screenshot review (S2.2)

## User workflow

Select a saved Pokémon using its existing collection review button. The detail view shows its UUID, current species, CP and IVs. **Evolve** loads available reference relationships without writing anything. Choose the actual target, inspect the transformation and CP invalidation, then confirm that it occurred in Pokémon GO. **Cancel evolution** discards the pending action. Successful confirmation updates the same individual and exposes **Update from Screenshot** for optional new observations.

**Update from Screenshot** captures the selected UUID and observation revision before opening the existing PNG/JPEG picker. The existing native extraction and bundled local OCR process the image. Review the extracted display label and each tentative confidence/method, compare saved and proposed values, and select **Update CP** and/or **Update appraisal IVs**. Neither option is selected automatically. Missing or unselected fields retain the current value. A selected field must be present; targeted review cannot clear a value by accident. Use the existing manual corrections to deliberately mark CP or IVs unknown.

Verify the same individual and its exact saved species/form, then confirm the update. OCR display labels may be nicknames and never update species or the saved display name. A differing label is surfaced for review. The species/form inputs start from the saved identity and must match it; a mismatch blocks all writes. Cancel and use the appropriate confirmed evolution or explicit saved identity correction first. When identity is unknown, both identity inputs stay empty and the user still confirms the individual. The application does not match against the collection.

Other record actions and individual selection are disabled during a pending lifecycle review. Recoverable save errors retain the draft. Conflicts, deleted individuals and already-confirmed requests require reloading authoritative inventory and starting a new review. Request IDs prevent repeating an action after a lost response. **Load confirmed history** shows completed evolution and observation events, not pending actions or strategic recommendations.

## Native contracts and integrity

- `evolution_options(id)` is read-only and returns expected strategic revision, catalog ID/version, explicit target species/form IDs and an unavailable reason when applicable.
- `confirm_evolution(id, input)` requires a unique request UUID, expected strategic observation revision, catalog ID/version and direct target form ID. Native code revalidates the target under an immediate transaction. Only catalog evolution edges to a different species are actionable; descriptive requirements are never executable eligibility rules.
- `reobserve_pokemon(id, input)` requires request UUID, expected strategic revision, explicit `cp`/`ivs` patches or null, exact `observedIdentity` or null and `sameIndividualConfirmed: true`. Null patches mean **keep**, not clear. A supplied patch contains its complete value and `screenshot` or `manual` source. Sources describe user-confirmed extraction versus user entry/edit, not measured recognition accuracy. No image, path, filename, OCR text or confidence score enters persistence.
- `pokemon_history(id)` returns confirmed request/individual IDs, before/after observations, patch provenance, reviewed identity, catalog context and authoritative UTC timestamps. Events are inserted atomically with the mutation; repeated IDs are rejected even for no-op observations.

Evolution preserves UUID, creation time, nickname, IVs, tags and appraisal state. CP becomes null because old species CP cannot establish resulting CP. Species-dependent moves, observed HP and level do not exist in this schema; none are copied, derived or fabricated. Future fields must define their invalidation semantics before joining the workflow. Existing correction behavior is retained. Previous observations survive in the event snapshot; the repository had no lifecycle history to reuse, so migration 7 adds only one table and its index/update guard.

The existing strategic observation revision covers actual CP/IV/identity changes only. Lifecycle history does not increment it, and appraisal revision is not reused. Evolution may advance the counter twice (CP invalidation and identity change). Unique request IDs independently prevent duplicate confirmed actions. All validation, observation writes, revision-trigger effects and history insertion commit or roll back together. Catalog version changes invalidate evolution drafts. Original screenshots are opened read-only, never deleted or retained; temporary preview URLs and OCR workers are released on cancel, success, failure and unmount.

## Reference and scope limits

The production catalog is still `tcm-empty` / `empty-1` pending verified dataset redistribution rights. Real Pokémon evolution is unavailable until a reviewed reference release supplies those relationships. Tests use hand-authored relationship fixtures, including named Charmander → Charmeleon → Charizard identities with artificial dex identifiers and no imported statistics; these are not a shipped or licensed game dataset. The existing reviewed adapter excludes Mega relationships. Schema-1 evolution edges are a release-reviewed assertion of normal permanent evolution, not a generic transformation graph; same-species edges are additionally excluded by the native action API. Fusion, trading and temporary transformations must never be encoded as evolution edges.

No spending, eligibility calculation, Power Up, recommendations, ML, additional gameplay actions, external API or cloud dependency is introduced. Screenshot extraction retains its existing bounded-layout and uncalibrated-confidence limitations. The user remains responsible for confirming the selected individual; the application cannot prove two screenshots depict the same Pokémon. History covers these two workflows, not prior corrections or every repository edit. Deleting an individual cascades its events through the existing delete contract.

## Verification and next increment

See the current [validation log](validation.md). Tests cover successive evolutions retaining one UUID, branches/invalid targets/cancellation, preservation, post-evolution unknown CP, opt-in screenshot patches, missing data/species mismatch, stale and repeated requests, transaction rollback and existing imports/corrections.

Suggested S2.3: one explicit selected player goal/context per individual, including unknown state and a narrowly scoped native revision. Keep moves, level, resources and mechanics expansion tied to that concrete goal; do not introduce strategic scoring or spending yet.
