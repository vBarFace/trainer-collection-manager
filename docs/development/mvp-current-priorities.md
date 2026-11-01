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
3. **Transfer-candidate review.** Assess reference-based role usefulness and
   potential, then owned alternatives and collection value. Explain why an individual is a candidate and what
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

## Assessment foundation and personalization

The user explicitly requires recommendations to draw on external reference facts
and suitable mathematical/statistical algorithms, not only the owned collection.
Use two distinct stages:

1. **Reference-based role assessment.** Assess the individual against verified
   mechanics and a declared gym/Rocket context, using species/form stats, types,
   legal moves, relevant opponents, and justified calculations or simulations.
   Where useful, benchmark against other eligible species/forms and move sets
   in the reference universe, including those the user does not own. Preserve
   current-versus-potential and known-versus-assumed distinctions.
2. **Personalized action priority.** Use the owned collection, resources, goals
   and protections to decide whether keeping, preparing or investing in this
   individual is worthwhile for this user. An owned alternative can lower its
   investment priority without changing its underlying assessed performance.

Reference-based assessment must remain useful when the collection contains only
this individual. A strong role assessment must not depend on the user owning weak
alternatives, and the best owned individual must not automatically be called a
strong performer relative to the reference benchmark. Avoid a single universal
quality score: attack, defense and Rocket require distinct contexts and evidence.

PvPoke, PokeMiners and PoGoAPI remain starting sources; select evidence per field
and gameplay mode. A provider's PvP ranking cannot stand in for gym or Rocket
performance. Additional datasets or published algorithms may be evaluated when
their mechanics, assumptions, freshness, provenance and permitted local use are
verified. Do not assert that a named source, algorithm or simulation is integrated
or validated before that work is executed. Keep acquired data local and private.

Here, models include mathematical damage/statistical models and deterministic
simulations. The existing no-ML/no-LLM MVP requirement remains in force; the user's
general reference to models/algorithms does not by itself authorize learned models.
Missing evidence yields limited or unsupported output rather than an invented
external ranking or an undisclosed assumption about the individual's moves/level.

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

## Recommendation presentation and individual roadmaps

The individual roadmap remains a core deliverable throughout the prioritized
gym, Rocket and transfer work. Extend the existing UUID-linked planner alongside
each role assessment, starting with the first gym increment; do not defer it
until after all roles or replace it with a role label alone.

Lead with understandable role guidance and useful opponent-type groups. Show
the individual's confirmed typing and distinguish it from its observed attack
types or a clearly labeled potential move set. Attack typing, dual types,
resistances, move availability and the relevant mechanics must inform the result;
the individual's species typing alone cannot establish a favorable matchup.

Use a declared set of representative scenarios for the default broad summary,
without requiring the user to configure one specific opponent per individual.
Include a few representative opponent species where useful, with exact matchup
details available on demand. Do not overwhelm the default view with exhaustive
counter lists or present type advantage alone as verified overall performance.
Defense summaries should explain durability and relevant weaknesses under the
declared assumptions rather than promise universal superiority. General matchup
guidance must remain distinct from verified simulation results.

Connect the explanation to the same individual's ordered next steps: use now,
optionally review relevant missing data, consider a supported evolution, adjust
moves or Power Up where justified, or retain/wait without investment. Show the
expected benefit, prerequisites, costs when known and uncertainty. Preserve
postpone/ignore, explicit action confirmation, history and stale-plan detection.
Do not require all optional observations before showing limited potential, or
mark unsupported investments executable. These are next-increment requirements,
not claims that the current collection/statistical roadmap already supports them.

## Next concrete increment and validation

Implement the smallest end-to-end **gym-attack** assessment using verified local
move/compatibility/type data and a stated target context, followed by gym defense.
Calculate reference-based performance first, then personalize the recommendation
using owned alternatives. Collection-only comparisons do not meet acceptance.
Expose the result in the existing recommendation/details views. Reuse the current
manual observations and pure Rust evaluation boundary; avoid rebuilding import,
inventory or planning infrastructure.
The same increment must connect the broad type-oriented role summary and optional
opponent examples to the individual's roadmap with only supported next steps.

Validate independently reviewed damage/matchup examples, move availability,
missing/contradictory observations, collection alternatives, protection, stale
results after confirmed actions or reference changes, and offline operation.
Verify that an individual can receive a role assessment without owned peers;
adding a stronger owned alternative changes action priority without changing the
same individual's performance calculation under identical reference/context inputs.
Check that weak reference-level performance is not relabeled strong merely for
being the best owned option, and that external rankings apply to the stated mode.
Verify that type summaries distinguish observed from proposed attacks, account
for dual-type exceptions, disclose limited evidence, and preserve UUID-linked
roadmap confirmation and invalidation through each supported role assessment.
Real provider validation remains private, with original fictional committed
fixtures and separate provenance/rights notices. Actual gameplay support may
only be claimed after implementation and executed verification.
