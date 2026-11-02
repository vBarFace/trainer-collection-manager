import type { GymAssessment } from "./gym-native";

function typeNames(values: string[]) {
  return values
    .map((value) => value.charAt(0).toUpperCase() + value.slice(1))
    .join(", ");
}

export function GymAssessmentView({
  assessment,
  onSelectPokemon,
}: {
  assessment: GymAssessment;
  onSelectPokemon?: (id: string) => void;
}) {
  const alternative = assessment.ownedAlternativePokemonId;
  return (
    <section aria-label="Gym attack potential">
      <h3>Gym attack potential</h3>
      <p>{assessment.summary}</p>
      <p>
        Reference-based potential; actual battle readiness and investments
        remain unverified.
      </p>
      <p>
        Pokémon typing: {typeNames(assessment.pokemonTypes) || "Unknown"}.
        Attack types depend on the move set shown below.
      </p>
      {assessment.vulnerableTo.length > 0 && (
        <p>
          Typing vulnerabilities: {typeNames(assessment.vulnerableTo)}. This
          describes typing, not a gym-defense rating.
        </p>
      )}
      {assessment.builds.length === 0 ? (
        <p>
          No supported attack build can be assessed with the current evidence.
        </p>
      ) : (
        assessment.builds.map((build) => (
          <article key={build.attackType}>
            <h4>{typeNames([build.attackType])} attack potential</h4>
            {build.referenceSummary && <p>{build.referenceSummary}</p>}
            <p>
              {build.fastMove} + {build.chargedMove}.
            </p>
            <p>
              {build.moveState === "confirmed"
                ? "Move names match your confirmed observations and the installed move pool. Current battle readiness is still unverified."
                : "Possible move set from the installed reference. These are proposed attacks, not confirmed observations of this individual."}
            </p>
            <p>
              Favorable type coverage:{" "}
              {typeNames(build.strongAgainst) || "None established"}.
            </p>
            {build.resistedBy.length > 0 && (
              <p>Reduced coverage against: {typeNames(build.resistedBy)}.</p>
            )}
            <p>
              These groups describe single-type targets. Dual typing and the
              opponent’s moves can change the matchup.
            </p>
            <details>
              <summary>
                Reference-model details for {typeNames([build.attackType])}
              </summary>
              <p>
                Idealized neutral energy-cycle estimate:{" "}
                {build.neutralCycleDps.toFixed(2)} modeled damage per second.
                This fixed stationary PvE model is not a battle simulation or
                actual measured DPS for this individual.
              </p>
              <p>
                Reference percentile: {build.referencePercentile.toFixed(1)}% ·{" "}
                {build.comparedForms} compared forms under the same model. This
                is not an overall Pokémon quality score.
              </p>
              {build.referenceBest.length > 0 && (
                <p>
                  Reference options: {build.referenceBest.join(", ")}. These
                  illustrate the benchmark, not opponent recommendations.
                  Release status and eligibility are not established by this
                  list.
                </p>
              )}
            </details>
          </article>
        ))
      )}
      {alternative && (
        <>
          <p>
            Another owned individual may lower the priority of investing here;
            it does not change this reference-based potential assessment.
          </p>
          {onSelectPokemon && (
            <button type="button" onClick={() => onSelectPokemon(alternative)}>
              Inspect owned gym-attack alternative
            </button>
          )}
        </>
      )}
      {assessment.missing.length > 0 && (
        <>
          <h4>Missing information</h4>
          <ul>
            {assessment.missing.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </>
      )}
      {assessment.limitations.length > 0 && (
        <details>
          <summary>Model assumptions and limitations</summary>
          <ul>
            {assessment.limitations.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </details>
      )}
      <p>
        Gym attack only. Gym defense and Team Rocket performance are not
        assessed here. Model: {assessment.rulesetVersion}.
      </p>
    </section>
  );
}
