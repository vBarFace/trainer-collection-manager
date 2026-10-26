import { useEffect, useRef } from "react";
import { evaluationLabels, type InventoryRecord } from "./native";

export function AppraisalPanel({
  record,
  busy,
  error,
  onEvaluate,
  onReload,
}: {
  record: InventoryRecord;
  busy: boolean;
  error: string;
  onEvaluate: () => void;
  onReload: () => void;
}) {
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    heading.current?.focus();
  }, [record.id]);
  const evaluation = record.evaluation;
  const result = evaluation.result;
  return (
    <section
      className="appraisal-panel"
      aria-labelledby="appraisal-title"
      aria-busy={busy}
    >
      <h3 id="appraisal-title" ref={heading} tabIndex={-1}>
        Appraisal for {record.displayName}
      </h3>
      <p>
        Scope: appraisal IVs only. Strategic evaluation has not been performed.
      </p>
      <p>Appraisal state: {evaluationLabels[evaluation.state]}</p>
      <p>Current ruleset: {evaluation.currentRulesetVersion}</p>
      {evaluation.evaluatedAt && (
        <p>
          Last appraisal attempt: {evaluation.evaluatedAt} UTC ·{" "}
          {evaluation.rulesetVersion}
        </p>
      )}
      {evaluation.state === "pending" && (
        <p>No appraisal evaluation has been requested.</p>
      )}
      {evaluation.state === "stale" && (
        <p>
          The confirmed IVs or ruleset changed. The previous result is withheld;
          re-evaluate to assess current data.
        </p>
      )}
      <button type="button" onClick={onEvaluate} disabled={busy}>
        {busy
          ? "Evaluating appraisal…"
          : error
            ? "Retry appraisal evaluation"
            : evaluation.state === "pending"
              ? "Evaluate appraisal"
              : "Re-evaluate appraisal"}
      </button>
      {busy && <p role="status">Evaluating confirmed appraisal IVs locally…</p>}
      {error && (
        <div role="alert">
          <p>{error}</p>
          <button type="button" onClick={onReload}>
            Reload inventory
          </button>
        </div>
      )}
      {result && (
        <div role="status" aria-live="polite">
          <h4>Evaluated data</h4>
          {result.ivs ? (
            <>
              <p>Confirmed attack IV: {result.ivs.attack} / 15</p>
              <p>Confirmed defense IV: {result.ivs.defense} / 15</p>
              <p>Confirmed HP IV: {result.ivs.hp} / 15</p>
              <p>Total appraisal IVs: {result.total} / 45</p>
              <p>
                Appraisal percentage: {result.appraisalPercentage?.toFixed(2)}%
              </p>
              <p>Perfect 15/15/15 appraisal: {result.perfect ? "Yes" : "No"}</p>
            </>
          ) : (
            <p>
              Insufficient data. Required confirmed IVs are unknown:{" "}
              {result.missingData
                .map((field) => (field === "hp" ? "HP" : field))
                .join(", ")}
              . No total or percentage can be calculated. Review confirmation
              accepts all three IVs together; editing existing inventory data is
              not available in this increment.
            </p>
          )}
          <h4>Appraisal findings</h4>
          {result.findings.length ? (
            result.findings.map((finding) => (
              <p key={finding.code}>{finding.explanation}</p>
            ))
          ) : (
            <p>No noteworthy appraisal finding.</p>
          )}
          <h4>Recommendations and suggested tags</h4>
          {result.recommendations.length ? (
            result.recommendations.map((recommendation) => (
              <p key={recommendation.code}>{recommendation.explanation}</p>
            ))
          ) : (
            <p>
              No action recommendations or suggested tags. Appraisal alone does
              not establish strategic value.
            </p>
          )}
          <h4>Limitations</h4>
          {result.limitations.map((limitation) => (
            <p key={limitation}>{limitation}</p>
          ))}
        </div>
      )}
      <h4>Assigned tags</h4>
      <p>
        {record.tags.length
          ? record.tags.map((tag) => tag.name).join(", ")
          : "None"}
        . Evaluation does not assign or remove tags.
      </p>
    </section>
  );
}
