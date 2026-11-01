# Current MVP priorities

Confirmed by the user on 2026-11-01. This project-scoped clarification supersedes
the earlier proposal to prioritize PvP or expand screenshot extraction next.
It changes the delivery plan, not the currently implemented gameplay coverage.

## Screenshot scope

- Automatically suggest only display name, CP and the three appraisal IVs.
- Keep all other supported individual fields manually entered and optional.
- Unknown values remain unknown; optional fields never block ordinary saving.
- Defer OCR for HP, moves, dimensions, dates, symbols and variants. Extending
  extraction is not a prerequisite for delivering the prioritized assessments.
- Retain explicit review, confidence/provenance, batch grouping, stable UUIDs
  and screenshot cleanup. Accuracy validation concerns the existing fields only.

## Delivery order

1. **Gym attack and defense.** Build the minimum verified move, availability,
   type-effectiveness and gym-mechanics reference support required by a bounded
   gym assessment. Deliver a gym-attack comparison first, then a distinct defense
   assessment with explicit eligibility, opponent and time assumptions. Do not
   insert raid features ahead of these goals merely because they share data.
2. **Team Rocket.** Reuse verified reference facts, while modeling the selected
   Rocket opponent/context separately. Distinguish general matchup guidance from
   actual simulated performance and from current readiness.
3. **Transfer-candidate review.** Compare owned alternatives across supported
   roles and collection value. Explain why an individual is a candidate and what
   still requires review. Preserve protection, favorites, unique/special forms,
   relevant special-move value and unresolved collection-value cases. Never
   recommend transfer solely from CP or IV percentage and never perform it.
4. **PvP expansion.** Only after the above flows: move suitability, legal league
   context and contextual competitive evidence. Existing theoretical IV/stat
   calculations remain available, but are not an overall battle assessment.

Transfer review must ship before expanded PvP. State clearly that competitive
PvP usefulness has not been fully assessed; an unassessed role is not evidence
of no value. Keep uncertain cases under review rather than certify universal
safe disposal. The workflow need not wait for a complete PvP engine.

## Behavior with optional manual inputs

Confirmed species/form plus reference facts can support a limited potential
assessment without pretending the individual already has a proposed move set.
An extracted display name may be a nickname and never silently confirms identity.
Missing observed moves, level or other required context limit current-readiness
and numerical claims; explain those limits and offer optional manual completion.
Do not make all manual fields mandatory to access the application or save a
Pokémon. Do not replace missing facts with assumed attacks, level or variants.

Each assessment must separate current use from possible preparation, identify
the modeled target/context, explain facts and uncertainty, compare relevant
owned alternatives, and record reference/mechanics versions. Unknown costs stay
unknown and block executable investments. Existing UUIDs, lifecycle confirmation,
bounded roadmap progress and local-only persistence remain the foundation.

## Next concrete increment and validation

Implement the smallest end-to-end **gym-attack** assessment using verified local
move/compatibility/type data and a stated target context, followed by gym defense.
Expose the result in the existing recommendation/details views. Reuse the current
manual observations and pure Rust evaluation boundary; avoid rebuilding import,
inventory or planning infrastructure.

Validate independently reviewed damage/matchup examples, move availability,
missing/contradictory observations, collection alternatives, protection, stale
results after confirmed actions or reference changes, and offline operation.
Real provider validation remains private, with original fictional committed
fixtures and separate provenance/rights notices. Actual gameplay support may
only be claimed after implementation and executed verification.
