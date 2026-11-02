import { useEffect, useRef, useState } from "react";
import { nativeErrorMessage } from "./native";
import { IndividualRoadmap } from "./IndividualRoadmap";
import { GymAssessmentView } from "./GymAssessmentView";
import { PreparationPlannerView } from "./PreparationPlannerView";
import { IndividualDecisionPanel } from "./IndividualDecisionView";
import {
  strategyNative,
  type CollectionStrategy,
  type StrategyReport,
} from "./strategy-native";

export function LeagueStatistics({
  report,
  onOpenIndividual,
}: {
  report: StrategyReport;
  onOpenIndividual: (id: string) => void;
}) {
  return (
    <section aria-label="Derived league statistics">
      <h3>Statistical potential</h3>
      <p>
        IV rank compares hypothetical spreads of the same form within the stated
        level ceiling. It does not rank battle performance. Current readiness
        and investment costs remain unverified.
      </p>
      {report.leagues.map((league) => (
        <div key={league.league}>
          <h4>
            {league.league}{" "}
            {league.cpLimit ? `· ${league.cpLimit} CP limit` : "· uncapped"}
          </h4>
          <p>{league.explanation}</p>
          {league.potential && (
            <p>
              Modeled ceiling {league.potential.levelCeiling} · hypothetical
              level {league.potential.level} · CP {league.potential.cp} · HP{" "}
              {league.potential.hp} · stat product{" "}
              {league.potential.statProduct.toFixed(1)} · IV rank{" "}
              {league.potential.ivRank} / {league.potential.rankedSpreads}
            </p>
          )}
          {league.alternativePokemonId && (
            <button
              type="button"
              onClick={() => onOpenIndividual(league.alternativePokemonId!)}
            >
              Inspect owned alternative for {league.league}
            </button>
          )}
        </div>
      ))}
    </section>
  );
}

export function Recommendations({
  onOpenIndividual,
  initialSelectedId = null,
}: {
  onOpenIndividual: (id: string) => void;
  initialSelectedId?: string | null;
}) {
  const [snapshot, setSnapshot] = useState<CollectionStrategy | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState("");
  const [selected, setSelected] = useState<string | null>(initialSelectedId);
  const [goal, setGoal] = useState("balanced");
  const [budget, setBudget] = useState("");
  const [limit, setLimit] = useState(100);
  const alive = useRef(false);
  const pending = useRef(false);
  const assessmentHeading = useRef<HTMLHeadingElement>(null);
  async function load() {
    setBusy(true);
    setError("");
    try {
      const next = await strategyNative.collection();
      if (alive.current) {
        setSnapshot(next);
        setGoal(next.preferences.goal);
        setBudget(next.preferences.stardustBudget?.toString() ?? "");
      }
    } catch (failure) {
      if (alive.current)
        setError(
          nativeErrorMessage(
            failure,
            "Unable to assess the local collection. Retry after checking reference data.",
          ),
        );
    } finally {
      if (alive.current) setBusy(false);
    }
  }
  useEffect(() => {
    alive.current = true;
    void load();
    return () => {
      alive.current = false;
    };
  }, []);
  async function savePreferences() {
    if (!snapshot || pending.current) return;
    if (
      budget !== "" &&
      (!/^(0|[1-9][0-9]{0,9})$/.test(budget) || Number(budget) > 4294967295)
    ) {
      setError(
        "Use a whole Stardust budget from 0 to 4294967295, or leave it unknown.",
      );
      return;
    }
    pending.current = true;
    setBusy(true);
    try {
      await strategyNative.savePreferences(
        snapshot.preferences.revision,
        goal,
        budget === "" ? null : Number(budget),
      );
      await load();
    } catch (failure) {
      setError(
        nativeErrorMessage(
          failure,
          "Preferences changed. Reload before saving again.",
        ),
      );
    } finally {
      pending.current = false;
      setBusy(false);
    }
  }
  const reports = (snapshot?.reports ?? [])
    .filter(
      (report) =>
        report.displayName
          .toLocaleLowerCase()
          .includes(query.toLocaleLowerCase()) &&
        (!kind || report.recommendations.some((item) => item.kind === kind)),
    )
    .sort(
      (a, b) =>
        (a.recommendations[0]?.priority ?? 99) -
          (b.recommendations[0]?.priority ?? 99) ||
        a.displayName.localeCompare(b.displayName),
    );
  const current = snapshot?.reports.find(
    (report) => report.pokemonId === selected,
  );
  useEffect(() => {
    if (current) {
      assessmentHeading.current?.focus();
      assessmentHeading.current?.scrollIntoView?.({ block: "start" });
    }
  }, [selected, snapshot?.context, current]);
  return (
    <div className="recommendations">
      <section>
        <h2>Explainable collection guidance</h2>
        <p>
          Derived locally from your confirmed observations and installed
          reference. No universal quality score, automatic tags, transfers or
          game actions.
        </p>
        <button type="button" disabled={busy} onClick={() => void load()}>
          Recalculate collection
        </button>
        {busy && <p role="status">Analyzing local collection…</p>}
        {error && <p role="alert">{error}</p>}
        {snapshot && (
          <>
            <details>
              <summary>Optional priorities and shared budget</summary>
              <p>
                Use suggestions are evaluated automatically across roles. These
                optional preferences adjust priorities; no role choice is
                required.
              </p>
              <fieldset disabled={busy}>
                <legend>Optional global goals and resources</legend>
                <label>
                  Collection goal
                  <select
                    value={goal}
                    onChange={(e) => setGoal(e.target.value)}
                  >
                    <option value="balanced">
                      Balanced collection (default)
                    </option>
                    <option value="collection">Collection completeness</option>
                    <option value="gym_attack">Gym attack preparation</option>
                    <option value="gym_defense">Gym defense preparation</option>
                    <option value="rocket">Team Rocket preparation</option>
                    <option value="great">Great League statistics</option>
                    <option value="ultra">Ultra League statistics</option>
                    <option value="master">Master League statistics</option>
                  </select>
                </label>
                <label>
                  Confirmed available Stardust budget
                  <input
                    value={budget}
                    inputMode="numeric"
                    onChange={(e) => setBudget(e.target.value)}
                  />
                </label>
                <p>
                  Blank means unknown; zero is a real constraint. Costs and
                  eligibility remain required before an investment can be
                  executable.
                </p>
                <button type="button" onClick={() => void savePreferences()}>
                  Confirm goals and budget
                </button>
              </fieldset>
            </details>
            <label>
              Search recommendations
              <input
                value={query}
                onChange={(e) => {
                  setQuery(e.target.value);
                  setLimit(100);
                }}
              />
            </label>
            <label>
              Recommendation type
              <select value={kind} onChange={(e) => setKind(e.target.value)}>
                <option value="">All guidance</option>
                {[
                  ...new Set(
                    snapshot.reports.flatMap((report) =>
                      report.recommendations.map((item) => item.kind),
                    ),
                  ),
                ]
                  .sort()
                  .map((value) => (
                    <option key={value}>{value}</option>
                  ))}
              </select>
            </label>
            <p>
              {reports.length} matching individuals. Priorities describe the
              next safe review or collection decision, not combat strength.
            </p>
            {snapshot.reports.length === 0 && (
              <p>
                Import and confirm individuals to receive collection guidance.
                Reference data adds species/form comparisons and supported
                statistics.
              </p>
            )}
            <ul className="inventory-list">
              {reports.slice(0, limit).map((report) => (
                <li key={report.pokemonId}>
                  <h3>{report.displayName}</h3>
                  <p>
                    Rules: {report.rulesetVersion} · reference:{" "}
                    {report.datasetId}/{report.datasetVersion}
                  </p>
                  <p>
                    {report.protected
                      ? "Retained by your confirmed constraints"
                      : "Review collection value before irreversible action"}{" "}
                    · same-form peers: {report.duplicateCount}
                  </p>
                  {report.gym && <p>{report.gym.summary}</p>}
                  {report.recommendations
                    .filter((item) => !kind || item.kind === kind)
                    .map((item) => (
                      <div key={item.id}>
                        <h4>{item.kind}</h4>
                        <p>{item.explanation}</p>
                        {item.facts.length > 0 && (
                          <ul aria-label="Supporting confirmed facts">
                            {item.facts.map((fact) => (
                              <li key={fact}>{fact}</li>
                            ))}
                          </ul>
                        )}
                        {item.missing.length > 0 && (
                          <p>Missing or uncertain: {item.missing.join("; ")}</p>
                        )}
                        <p>
                          {item.executable
                            ? "Available within the stated scope"
                            : "Conditional: prerequisites are not verified"}
                        </p>
                        {item.alternativePokemonId && (
                          <button
                            type="button"
                            onClick={() =>
                              onOpenIndividual(item.alternativePokemonId!)
                            }
                          >
                            Inspect alternative individual
                          </button>
                        )}
                      </div>
                    ))}
                  <button
                    type="button"
                    onClick={() => setSelected(report.pokemonId)}
                  >
                    Statistics and assessment for {report.displayName}
                  </button>
                  <button
                    type="button"
                    onClick={() => onOpenIndividual(report.pokemonId)}
                  >
                    Review confirmed facts for {report.displayName}
                  </button>
                </li>
              ))}
            </ul>
            {reports.length > limit && (
              <button
                type="button"
                onClick={() => setLimit((count) => count + 100)}
              >
                Show 100 more recommendations
              </button>
            )}
          </>
        )}
      </section>
      {current && (
        <section className="individual-assessment">
          <h2 ref={assessmentHeading} tabIndex={-1}>
            {current.displayName} · individual assessment
          </h2>
          {current.decision && (
            <IndividualDecisionPanel
              key={`${current.pokemonId}:${snapshot?.context}`}
              id={current.pokemonId}
              observationRevision={current.observationRevision}
              refreshToken={0}
              onOpenIndividual={onOpenIndividual}
            />
          )}
          {current.gym && (
            <GymAssessmentView
              assessment={current.gym}
              onSelectPokemon={onOpenIndividual}
            />
          )}
          <details>
            <summary>Optional league statistics</summary>
            <LeagueStatistics
              report={current}
              onOpenIndividual={onOpenIndividual}
            />
          </details>
          {!current.decision &&
            (current.preparation || current.roles?.length) && (
              <PreparationPlannerView
                key={current.pokemonId + ":" + snapshot?.context}
                pokemonId={current.pokemonId}
                expectedContext={snapshot?.context}
                plan={current.preparation}
                roles={current.decision ? [] : current.roles}
                onOpenIndividual={onOpenIndividual}
                onChanged={() => void load()}
              />
            )}
          <details>
            <summary>Evidence, versions and limitations</summary>
            <p>
              {current.rulesetVersion} · observation revision{" "}
              {current.observationRevision} · {current.datasetId}/
              {current.datasetVersion} · mechanics{" "}
              {current.mechanicsVersion ?? "Unknown"}
            </p>
            <p className="digest">
              Mechanics digest: {current.mechanicsSha256 ?? "Unavailable"}
            </p>
            <ul>
              {current.limitations.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </details>
          {!current.decision && (
            <IndividualRoadmap
              key={`${current.pokemonId}:${snapshot?.context}`}
              id={current.pokemonId}
              onOpenIndividual={onOpenIndividual}
            />
          )}
        </section>
      )}
    </div>
  );
}
