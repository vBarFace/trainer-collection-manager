import { useEffect, useRef, useState } from "react";
import { nativeErrorMessage } from "./native";
import {
  preparationNative,
  type PreparationPlan,
  type ResourceBudget,
  type RoleAssessment,
  type PlanConflict,
} from "./preparation-native";

const labels: Record<string, string> = {
  gym_attack: "Gym attack",
  gym_defense: "Gym defense",
  rocket: "Team Rocket",
  pvp: "PvP",
  transfer: "Transfer review",
  use_now: "Use as is",
  power_up: "Power Up",
  evolve: "Evolution",
  change_moves: "Moves",
  wait: "Wait / retain",
  within_budget: "Within the confirmed budget",
  over_budget: "Exceeds the confirmed budget",
  unknown: "Resources not fully confirmed",
};
function label(value: string) {
  return labels[value] ?? value.replaceAll("_", " ");
}
function amount(value: number | null) {
  return value === null ? "Unknown" : value.toLocaleString("en-GB");
}

function ConflictWarnings({ conflicts }: { conflicts: PlanConflict[] }) {
  return (
    <ul>
      {conflicts.map((conflict, index) => (
        <li key={`${conflict.pathId}:${conflict.role}:${index}`}>
          {conflict.severity === "blocked" ? "Blocked" : "Review"} ·{" "}
          {label(conflict.role)}: {conflict.explanation}
        </li>
      ))}
    </ul>
  );
}

function ResourceEditor({
  id,
  initial,
  onChanged,
}: {
  id: string;
  initial?: ResourceBudget;
  onChanged?: () => void;
}) {
  const [budget, setBudget] = useState<ResourceBudget | null>(initial ?? null);
  const [candy, setCandy] = useState(initial?.candy?.toString() ?? "");
  const [candyXl, setCandyXl] = useState(initial?.candyXl?.toString() ?? "");
  const [stardust, setStardust] = useState(initial?.stardust?.toString() ?? "");
  const [maximumLevel, setMaximumLevel] = useState(
    initial?.maximumLevel?.toString() ?? "",
  );
  const [normal, setNormal] = useState(initial?.normalCostsConfirmed ?? false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [saved, setSaved] = useState(false);
  const pending = useRef(false);
  useEffect(() => {
    if (initial) return;
    let live = true;
    void preparationNative
      .resources(id)
      .then((next) => {
        if (!live) return;
        setBudget(next);
        setCandy(next.candy?.toString() ?? "");
        setCandyXl(next.candyXl?.toString() ?? "");
        setStardust(next.stardust?.toString() ?? "");
        setMaximumLevel(next.maximumLevel?.toString() ?? "");
        setNormal(next.normalCostsConfirmed);
      })
      .catch((failure: unknown) => {
        if (live)
          setError(
            nativeErrorMessage(failure, "Unable to load planning resources."),
          );
      });
    return () => {
      live = false;
    };
  }, [id, initial]);
  async function save() {
    if (!budget || pending.current) return;
    if (
      [candy, candyXl, stardust].some(
        (value) =>
          value !== "" &&
          (!/^(0|[1-9][0-9]{0,9})$/.test(value) || Number(value) > 4294967295),
      )
    ) {
      setError(
        "Use whole resource amounts from 0 to 4294967295, or leave them unknown.",
      );
      return;
    }
    if (
      maximumLevel !== "" &&
      (!Number.isFinite(Number(maximumLevel)) ||
        Number(maximumLevel) < 1 ||
        Number(maximumLevel) > 50 ||
        (Number(maximumLevel) * 2) % 1 !== 0)
    ) {
      setError(
        "Use an in-game level cap from 1 to 50 in half-level increments, or leave it unknown.",
      );
      return;
    }
    pending.current = true;
    setBusy(true);
    setError("");
    setSaved(false);
    try {
      const next = await preparationNative.saveResources(id, budget.revision, {
        candy: candy === "" ? null : Number(candy),
        candyXl: candyXl === "" ? null : Number(candyXl),
        stardust: stardust === "" ? null : Number(stardust),
        maximumLevel: maximumLevel === "" ? null : Number(maximumLevel),
        normalCostsConfirmed: normal,
        selectedPathId: budget.selectedPathId ?? null,
      });
      setBudget(next);
      setSaved(true);
      onChanged?.();
    } catch (failure) {
      setError(
        nativeErrorMessage(
          failure,
          "Planning resources changed. Reload before saving again.",
        ),
      );
    } finally {
      pending.current = false;
      setBusy(false);
    }
  }
  return (
    <details>
      <summary>Optional planning resources for this individual</summary>
      <p>
        These are allocations for this plan. Candy may be shared by an evolution
        family and Stardust by your collection; saving an allocation does not
        reserve or spend either balance. Blank means unknown; zero is a known
        limit.
      </p>
      {error && <p role="alert">{error}</p>}
      {saved && (
        <p role="status">
          Planning resources saved. Recalculate to refresh costs and
          prerequisites.
        </p>
      )}
      <fieldset disabled={busy || budget === null}>
        <legend>Manually confirmed allocation</legend>
        <label>
          Candy allocation
          <input
            value={candy}
            inputMode="numeric"
            onChange={(e) => setCandy(e.target.value)}
          />
        </label>
        <label>
          Candy XL allocation
          <input
            value={candyXl}
            inputMode="numeric"
            onChange={(e) => setCandyXl(e.target.value)}
          />
        </label>
        <label>
          Stardust allocation
          <input
            value={stardust}
            inputMode="numeric"
            onChange={(e) => setStardust(e.target.value)}
          />
        </label>
        <label>
          Confirmed in-game Power Up level cap
          <input
            value={maximumLevel}
            inputMode="decimal"
            onChange={(e) => setMaximumLevel(e.target.value)}
          />
        </label>
        <label>
          <input
            type="checkbox"
            checked={normal}
            onChange={(e) => setNormal(e.target.checked)}
          />
          I confirmed standard costs apply (not Lucky, Shadow or Purified)
        </label>
        <button type="button" onClick={() => void save()}>
          Confirm planning allocation
        </button>
      </fieldset>
    </details>
  );
}

export function PreparationPlannerView({
  pokemonId,
  plan,
  roles = [],
  onOpenIndividual,
  onChanged,
  expectedContext,
  stale = false,
}: {
  pokemonId: string;
  plan?: PreparationPlan | null;
  roles?: RoleAssessment[];
  onOpenIndividual: (id: string) => void;
  onChanged?: () => void;
  expectedContext?: string;
  stale?: boolean;
}) {
  const [selecting, setSelecting] = useState(false);
  const [selectionError, setSelectionError] = useState("");
  const [selection, setSelection] = useState<{
    pokemonId: string;
    context: string;
    pathId: string | null;
  } | null>(null);
  const localSelection =
    selection?.pokemonId === pokemonId && selection.context === expectedContext
      ? selection
      : null;
  const selectedPath = localSelection
    ? localSelection.pathId
    : (plan?.selectedPathId ?? plan?.resources?.selectedPathId ?? null);
  const selectionPending = useRef(false);
  const activePathId = selectedPath ?? plan?.primaryPathId;
  const activeConflicts = (plan?.conflicts ?? []).filter(
    (conflict) => conflict.pathId === activePathId,
  );
  const otherConflicts = (plan?.conflicts ?? []).filter(
    (conflict) => conflict.pathId !== activePathId,
  );
  async function select(pathId: string | null) {
    if (!expectedContext || stale || selectionPending.current) return;
    selectionPending.current = true;
    setSelecting(true);
    setSelectionError("");
    try {
      if (pathId === null)
        await preparationNative.clearRoute(pokemonId, expectedContext);
      else
        await preparationNative.selectPath(pokemonId, expectedContext, pathId);
      setSelection({ pokemonId, context: expectedContext, pathId });
      onChanged?.();
    } catch (failure) {
      setSelectionError(
        nativeErrorMessage(
          failure,
          "The planning context changed. Refresh the assessment before changing the roadmap route.",
        ),
      );
    } finally {
      selectionPending.current = false;
      setSelecting(false);
    }
  }
  return (
    <section aria-label="Preparation and role decisions">
      <h3>Roles and preparation options</h3>
      {roles.length > 0 && (
        <section aria-label="Role assessments">
          <p>
            An individual can serve several roles. Missing evidence remains
            unknown and does not establish that it is safe to transfer.
          </p>
          {roles.map((role) => (
            <article key={role.role}>
              <h4>{label(role.role)}</h4>
              <p>{role.summary}</p>
              <p>Assessment: {label(role.state)}.</p>
              {role.missing.length > 0 && (
                <p>Still needed: {role.missing.join("; ")}.</p>
              )}
              {role.reasons.length > 0 && (
                <details>
                  <summary>Evidence for {label(role.role)}</summary>
                  <ul>
                    {role.reasons.map((reason, index) => (
                      <li key={index}>{reason}</li>
                    ))}
                  </ul>
                </details>
              )}
            </article>
          ))}
        </section>
      )}
      {plan && (
        <>
          <p>{plan.summary}</p>
          {plan.planningMode === "automatic_multi_role" && (
            <p>
              The app considers supported uses together and proposes a route
              automatically. Alternative paths remain available for review.
            </p>
          )}
          {activeConflicts.length > 0 && (
            <section aria-label="Conflicts between possible uses">
              <h4>Review this route’s conflicts before investing</h4>
              <ConflictWarnings conflicts={activeConflicts} />
            </section>
          )}
          {otherConflicts.length > 0 && (
            <details>
              <summary>
                Conflicts for other alternatives ({otherConflicts.length})
              </summary>
              <p>
                These warnings apply to other preparation routes. Review them
                before switching plans.
              </p>
              <ConflictWarnings conflicts={otherConflicts} />
            </details>
          )}
          {selectionError && <p role="alert">{selectionError}</p>}
          {selectedPath && (
            <button
              type="button"
              disabled={stale || selecting || !expectedContext}
              onClick={() => void select(null)}
            >
              Use the app’s automatic plan
            </button>
          )}
          {localSelection && (
            <p role="status">
              {localSelection.pathId === null
                ? onChanged
                  ? "Manual route cleared. Updating the app’s automatic plan…"
                  : "Manual route cleared. Recalculate the roadmap to use the app’s automatic plan."
                : onChanged
                  ? "Plan selected. Updating the assessment and roadmap…"
                  : "Plan selected. Recalculate the roadmap to use its ordered steps."}
            </p>
          )}
          <p>
            Confirmed current level: {plan.currentLevel ?? "Unknown"}. Costs are
            estimates from the installed reference and your confirmed starting
            facts. These options do not record spending or game actions.
          </p>
          {stale && (
            <p role="status">
              These saved preparation options are stale. Recalculate the roadmap
              before acting on them.
            </p>
          )}
          {plan.paths.length === 0 && (
            <p>No preparation path is supported by the current reference.</p>
          )}
          {plan.paths.map((path) => (
            <article
              key={path.id}
              aria-label={`Preparation option for ${path.targetName}${path.targetLevel === null ? " at an unknown level" : ` at level ${path.targetLevel}`}`}
            >
              <details
                open={plan.paths.length === 1 || activePathId === path.id}
              >
                <summary>
                  <h4>
                    {path.targetName}
                    {path.targetLevel !== null &&
                      " · level " + path.targetLevel}
                    {" · " + label(path.role)}
                  </h4>
                  Candy: {amount(path.cost.candy)} · Candy XL:{" "}
                  {amount(path.cost.candyXl)} · Stardust:{" "}
                  {amount(path.cost.stardust)}
                </summary>
                <p>{path.benefit}</p>
                {plan.primaryPathId === path.id && (
                  <p>App-recommended preparation route</p>
                )}
                {path.servedRoles && path.servedRoles.length > 0 && (
                  <p>
                    Potential compatible uses:{" "}
                    {path.servedRoles.map(label).join(", ")}. These labels do
                    not establish current readiness.
                  </p>
                )}
                {path.projectedCp !== null &&
                  path.projectedCp !== undefined && (
                    <p>
                      Projected target CP: {path.projectedCp}. Confirm the
                      actual CP after the in-game action.
                    </p>
                  )}
                {selectedPath === path.id && <p>Selected roadmap plan</p>}
                {path.cost.items.length > 0 && (
                  <p>Items: {path.cost.items.join(", ")}.</p>
                )}
                <p>
                  {label(path.affordability)} ·{" "}
                  {path.executable && !stale
                    ? "Prerequisites supported within the stated model"
                    : "Conditional plan"}
                  .
                </p>
                {path.actions.length > 0 && (
                  <ol>
                    {path.actions.map((action, index) => (
                      <li key={index}>
                        <strong>{label(action.kind)}:</strong>{" "}
                        {action.description}
                        {action.acquisition.length > 0 && (
                          <p>How to obtain: {action.acquisition.join("; ")}.</p>
                        )}
                        {action.requirements.length > 0 && (
                          <p>Requirements: {action.requirements.join("; ")}.</p>
                        )}
                      </li>
                    ))}
                  </ol>
                )}
                {path.blockers.length > 0 && (
                  <p>Before proceeding: {path.blockers.join("; ")}.</p>
                )}
                {path.futureRoles && path.futureRoles.length > 0 && (
                  <details>
                    <summary>Other roles after preparation</summary>
                    {path.futureRoles.map((role) => (
                      <div key={role.role}>
                        <h5>{label(role.role)}</h5>
                        <p>{role.summary}</p>
                        {role.reasons.length > 0 && (
                          <p>{role.reasons.join("; ")}</p>
                        )}
                        {role.missing.length > 0 && (
                          <p>Still needed: {role.missing.join("; ")}</p>
                        )}
                      </div>
                    ))}
                  </details>
                )}
                <button
                  type="button"
                  disabled={
                    stale ||
                    selecting ||
                    !expectedContext ||
                    selectedPath === path.id
                  }
                  onClick={() => void select(path.id)}
                >
                  Use this plan in roadmap
                </button>
                <button
                  type="button"
                  disabled={stale}
                  onClick={() => onOpenIndividual(pokemonId)}
                >
                  Review facts and actions for this option
                </button>
              </details>
            </article>
          ))}
          {plan.missing.length > 0 && (
            <p>Missing planning inputs: {plan.missing.join("; ")}.</p>
          )}
          {plan.limitations.length > 0 && (
            <details>
              <summary>Preparation assumptions and limitations</summary>
              <ul>
                {plan.limitations.map((limitation, index) => (
                  <li key={index}>{limitation}</li>
                ))}
              </ul>
            </details>
          )}
          {!stale && (
            <ResourceEditor
              key={pokemonId + ":" + (plan.resources?.revision ?? "load")}
              id={pokemonId}
              initial={plan.resources}
              onChanged={onChanged}
            />
          )}
        </>
      )}
    </section>
  );
}
