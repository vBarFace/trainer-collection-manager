import { useEffect, useState } from "react";
import { nativeErrorMessage } from "./native";
import {
  individualDecisionNative,
  type IndividualDecision,
  type IndividualDecisionResult,
  type SuggestedUse,
} from "./decision-native";
import { IndividualRoadmap } from "./IndividualRoadmap";

const stateLabels: Record<string, string> = {
  supported_current: "Current use supported",
  preparation_potential: "Potential after preparation",
  needs_evidence: "More evidence needed",
  collection_value: "Keep for collection",
  retention_review: "Keep pending retention review",
};

function UseSuggestions({
  suggestions,
  label,
}: {
  suggestions: SuggestedUse[];
  label: string;
}) {
  return (
    <ul aria-label={label}>
      {suggestions.map((suggestion) => (
        <li key={suggestion.id}>
          <h4>
            {suggestion.tag} ·{" "}
            {stateLabels[suggestion.state] ??
              suggestion.state.replaceAll("_", " ")}
          </h4>
          {suggestion.targetName && (
            <p>Evaluated form: {suggestion.targetName}.</p>
          )}
          <p>{suggestion.explanation}</p>
          {suggestion.missing.length > 0 && (
            <p>Still needed: {suggestion.missing.join("; ")}.</p>
          )}
          {suggestion.evidence.length > 0 && (
            <details>
              <summary>Why this {suggestion.tag} suggestion?</summary>
              <ul>
                {suggestion.evidence.map((evidence, index) => (
                  <li key={index}>{evidence}</li>
                ))}
              </ul>
            </details>
          )}
        </li>
      ))}
    </ul>
  );
}

export function IndividualDecisionView({
  decision,
}: {
  decision: IndividualDecision;
}) {
  const qualifiedStates = new Set([
    "supported_current",
    "preparation_potential",
    "collection_value",
  ]);
  const qualified = decision.suggestions.filter((suggestion) =>
    qualifiedStates.has(suggestion.state),
  );
  const unresolved = decision.suggestions.filter(
    (suggestion) => !qualifiedStates.has(suggestion.state),
  );
  const hasBattleUses = qualified.some(
    (suggestion) =>
      suggestion.state === "supported_current" ||
      suggestion.state === "preparation_potential",
  );
  return (
    <section aria-label="Suggested uses and tags">
      <h3>Suggested uses and tags</h3>
      <p>{decision.summary}</p>
      <p>
        These suggestions are separate from your applied tags. Current use,
        future potential and missing evidence are shown independently. Your
        applied tags and protection are preserved.
      </p>
      {qualified.length === 0 ? (
        <p>
          No role tag is established by the available evidence. Keep this
          individual for review rather than infer that it has no value.
        </p>
      ) : (
        <UseSuggestions
          suggestions={qualified}
          label="Suggested roles and retention"
        />
      )}
      {unresolved.length > 0 && (
        <details open={!hasBattleUses}>
          <summary>Other uses under review</summary>
          <p>
            These uses need further evidence or retention review. They are not
            established role recommendations.
          </p>
          <UseSuggestions
            suggestions={unresolved}
            label="Uses requiring further review"
          />
        </details>
      )}
      {decision.missing.length > 0 && (
        <p>
          Review before relying on this assessment:{" "}
          {decision.missing.join("; ")}.
        </p>
      )}
    </section>
  );
}

/** Assessment failures are separate from confirmed observation/action saves. */
export function IndividualDecisionPanel({
  id,
  observationRevision,
  refreshToken,
  onOpenIndividual,
}: {
  id: string;
  observationRevision: string;
  refreshToken: number;
  onOpenIndividual: (id: string) => void;
}) {
  const [attempt, setAttempt] = useState(0);
  const [result, setResult] = useState<{
    key: string;
    value: IndividualDecisionResult;
  } | null>(null);
  const [failure, setFailure] = useState<{
    key: string;
    message: string;
  } | null>(null);
  const requestKey = `${id}:${observationRevision}:${refreshToken}:${attempt}`;
  useEffect(() => {
    let live = true;
    void Promise.resolve()
      .then(() => individualDecisionNative.evaluate(id))
      .then((value) => {
        if (live) {
          if (
            value.report.pokemonId !== id ||
            (value.roadmap.report !== null &&
              value.roadmap.report.pokemonId !== id)
          ) {
            setFailure({
              key: requestKey,
              message:
                "The assessment did not match this individual. Reload recommendations.",
            });
            return;
          }
          setResult({ key: requestKey, value });
        }
      })
      .catch((error: unknown) => {
        if (live)
          setFailure({
            key: requestKey,
            message: nativeErrorMessage(
              error,
              "Unable to refresh recommendations and roadmap. Your confirmed Pokémon data is saved; retry the assessment separately.",
            ),
          });
      });
    return () => {
      live = false;
    };
  }, [id, requestKey]);
  const current = result?.key === requestKey ? result.value : null;
  const error = failure?.key === requestKey ? failure.message : null;
  return (
    <section aria-label="Individual decision and progression">
      <h2>Individual recommendations and progression</h2>
      {error ? (
        <>
          <p role="status">Recommendation refresh unavailable: {error}</p>
          <button
            type="button"
            onClick={() => setAttempt((value) => value + 1)}
          >
            Retry individual assessment
          </button>
        </>
      ) : !current ? (
        <p role="status">
          Updating suggested uses and roadmap from current confirmed data…
        </p>
      ) : (
        <>
          {current.notice && <p role="status">{current.notice}</p>}
          {current.report.decision ? (
            <IndividualDecisionView decision={current.report.decision} />
          ) : (
            <p>
              Integrated use suggestions are unavailable for this report. Review
              the current evidence before deciding.
            </p>
          )}
          <button
            type="button"
            onClick={() => setAttempt((value) => value + 1)}
          >
            Refresh this individual’s assessment
          </button>
          <button type="button" onClick={() => onOpenIndividual(id)}>
            Review species and optional facts
          </button>
          <IndividualRoadmap
            key={`${id}:${current.context}`}
            id={id}
            initialView={current.roadmap}
            onReassess={() => setAttempt((value) => value + 1)}
            onOpenIndividual={onOpenIndividual}
          />
        </>
      )}
    </section>
  );
}
