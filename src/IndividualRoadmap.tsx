import { useEffect, useRef, useState } from "react";
import { nativeErrorMessage } from "./native";
import { strategyNative, type RoadmapView } from "./strategy-native";

export function IndividualRoadmap({
  id,
  onOpenIndividual,
}: {
  id: string;
  onOpenIndividual: (id: string) => void;
}) {
  const [view, setView] = useState<RoadmapView | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const pending = useRef(false);
  async function reload() {
    if (pending.current) return;
    pending.current = true;
    setBusy(true);
    setError("");
    try {
      setView(await strategyNative.roadmap(id));
    } catch (failure) {
      setError(nativeErrorMessage(failure, "Unable to reload roadmap."));
    } finally {
      pending.current = false;
      setBusy(false);
    }
  }
  useEffect(() => {
    let live = true;
    void strategyNative
      .roadmap(id)
      .then((next) => {
        if (live) setView(next);
      })
      .catch((failure: unknown) => {
        if (live)
          setError(nativeErrorMessage(failure, "Unable to load roadmap."));
      });
    return () => {
      live = false;
    };
  }, [id]);
  async function act(stepId?: string, status?: string) {
    if (!view || pending.current) return;
    pending.current = true;
    setBusy(true);
    setError("");
    try {
      const next =
        stepId && status
          ? await strategyNative.progress(
              id,
              view.context,
              view.revision,
              stepId,
              status,
            )
          : await strategyNative.generate(id, view.currentContext);
      setView(next);
      if (status === "review_requested") onOpenIndividual(id);
    } catch (failure) {
      setError(
        nativeErrorMessage(
          failure,
          "The roadmap changed. Reload recommendations before trying again.",
        ),
      );
    } finally {
      pending.current = false;
      setBusy(false);
    }
  }
  return (
    <section aria-labelledby="roadmap-title">
      <h3 id="roadmap-title">Individual progression roadmap</h3>
      <p>UUID: {id}</p>
      <p>
        Progress records your decisions. Reviewing a step opens the individual
        confirmation workflow; a progress button never changes observed CP,
        species, moves or resources.
      </p>
      {error && (
        <>
          <p role="alert">{error}</p>
          <button type="button" disabled={busy} onClick={() => void reload()}>
            Reload roadmap
          </button>
        </>
      )}
      {!view ? (
        <p role="status">Loading roadmap…</p>
      ) : (
        <>
          {view.stale && (
            <p role="status">
              This saved roadmap is stale. Input, collection, preferences,
              action history or reference context changed. Recalculate before
              recording more progress.
            </p>
          )}
          <button type="button" disabled={busy} onClick={() => void act()}>
            {view.report
              ? "Recalculate roadmap"
              : "Create personalized roadmap"}
          </button>
          {view.report && (
            <>
              <p>
                Generated {view.generatedAt} UTC · {view.report.rulesetVersion}{" "}
                · progress revision {view.revision}
              </p>
              {view.report.gym && view.report.gym.builds.length > 0 && (
                <p>
                  Gym preparation focus:{" "}
                  {view.report.gym.builds
                    .map(
                      (build) =>
                        `${build.attackType} coverage with ${build.fastMove} + ${build.chargedMove}`,
                    )
                    .join("; ")}
                  . These are reference-based possibilities, not confirmed
                  battles or investments.
                </p>
              )}
              <ol>
                {view.report.steps.map((step) => (
                  <li key={step.id}>
                    <h4>
                      {step.required ? "Required" : "Optional"} · {step.action}
                    </h4>
                    <p>{step.rationale}</p>
                    <p>
                      Cost: {step.cost ?? "Unknown / no cost estimate"} ·
                      prerequisites: {step.prerequisites.join(", ") || "None"}
                    </p>
                    <p>
                      Progress: {view.progress[step.id] ?? "pending"} ·{" "}
                      {step.executable && !view.stale
                        ? "Available within stated scope"
                        : "Blocked"}
                    </p>
                    <fieldset disabled={busy || view.stale}>
                      <legend>Step progress</legend>
                      <button
                        type="button"
                        disabled={!step.executable}
                        onClick={() =>
                          void act(
                            step.id,
                            step.action === "review"
                              ? "review_requested"
                              : "completed",
                          )
                        }
                      >
                        I completed this
                      </button>
                      <button
                        type="button"
                        onClick={() => void act(step.id, "postponed")}
                      >
                        Postpone
                      </button>
                      <button
                        type="button"
                        onClick={() => void act(step.id, "ignored")}
                      >
                        Ignore
                      </button>
                      <button
                        type="button"
                        onClick={() => void act(step.id, "pending")}
                      >
                        Reset progress
                      </button>
                    </fieldset>
                  </li>
                ))}
              </ol>
              {view.previous && (
                <details>
                  <summary>Previous roadmap retained for context</summary>
                  <p>
                    {view.previous.generatedAt} UTC ·{" "}
                    {Object.entries(view.previous.progress)
                      .map(([step, state]) => `${step}: ${state}`)
                      .join("; ") || "No progress recorded"}
                  </p>
                  <p>Previous suggestions are historical and cannot execute.</p>
                </details>
              )}
            </>
          )}
        </>
      )}
    </section>
  );
}
