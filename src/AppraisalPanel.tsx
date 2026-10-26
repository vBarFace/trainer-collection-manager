import { useEffect, useRef, useState } from "react";
import {
  evaluationLabels,
  type AppraisalIvs,
  type InventoryRecord,
} from "./native";
import { IvCorrectionForm } from "./IvCorrectionForm";

export function AppraisalPanel({
  record,
  busy,
  error,
  onEvaluate,
  onReload,
  savingIvs,
  onSaveIvs,
}: {
  record: InventoryRecord;
  busy: boolean;
  error: string;
  onEvaluate: () => void;
  onReload: () => void;
  savingIvs: boolean;
  onSaveIvs: (
    expected: AppraisalIvs | null,
    corrected: AppraisalIvs | null,
  ) => Promise<void>;
}) {
  const heading = useRef<HTMLHeadingElement>(null);
  const correctionButton = useRef<HTMLButtonElement>(null);
  const [editing, setEditing] = useState(false);
  const [correctionStatus, setCorrectionStatus] = useState("");
  useEffect(() => {
    heading.current?.focus();
  }, [record.id]);
  useEffect(() => {
    if (!editing && correctionStatus) correctionButton.current?.focus();
  }, [editing, correctionStatus]);
  const evaluation = record.evaluation;
  const result = evaluation.result;
  return (
    <section
      className="appraisal-panel"
      aria-labelledby="appraisal-title"
      aria-busy={busy || savingIvs}
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
      <button
        ref={correctionButton}
        type="button"
        disabled={busy || savingIvs || editing}
        onClick={() => {
          setCorrectionStatus("");
          setEditing(true);
        }}
      >
        Correct saved IVs
      </button>
      {editing && (
        <IvCorrectionForm
          ivs={record.ivs}
          busy={savingIvs}
          onSave={onSaveIvs}
          onSaved={() => {
            setEditing(false);
            setCorrectionStatus(
              "IV correction saved. Re-evaluate separately if the appraisal is stale.",
            );
          }}
          onCancel={() => {
            setEditing(false);
            setCorrectionStatus("IV correction cancelled. Nothing was saved.");
          }}
          onReload={() => {
            setEditing(false);
            setCorrectionStatus("");
            onReload();
          }}
        />
      )}
      {correctionStatus && <p role="status">{correctionStatus}</p>}
      <button
        type="button"
        onClick={onEvaluate}
        disabled={busy || savingIvs || editing}
      >
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
          <button
            type="button"
            onClick={onReload}
            disabled={savingIvs || editing}
          >
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
              . No total or percentage can be calculated. Use Correct saved IVs
              to confirm all three values, then request appraisal evaluation
              again.
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
