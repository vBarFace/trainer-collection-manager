import { useEffect, useRef, useState } from "react";
import type { FormEvent } from "react";
import {
  native,
  nativeErrorMessage,
  evaluationLabels,
  type AppraisalIvs,
  type InventoryRecord,
  type ReferenceIdentity,
  type InventorySnapshot,
  type ScreenshotExtraction,
} from "./native";
import { emptyReview, validateReview, type ReviewFields } from "./review";
import { extractText } from "./ocr";
import { AppraisalPanel } from "./AppraisalPanel";
import { ReferencePanel } from "./ReferencePanel";
import { CpCorrectionPanel } from "./CpCorrectionPanel";
import {
  EvolutionPanel,
  ReobservationPanel,
  HistoryPanel,
} from "./LifecyclePanel";

interface Preview {
  url: string;
  width: number;
  height: number;
  extraction: ScreenshotExtraction;
}

export function Inventory() {
  const [snapshot, setSnapshot] = useState<InventorySnapshot | null>(null);
  const [loadError, setLoadError] = useState("");
  const [preview, setPreview] = useState<Preview | null>(null);
  const [fields, setFields] = useState<ReviewFields>(emptyReview);
  const [edited, setEdited] = useState<
    Partial<Record<keyof ReviewFields, boolean>>
  >({});
  const [phase, setPhase] = useState<
    "idle" | "selecting" | "extracting" | "saving"
  >("idle");
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [evaluating, setEvaluating] = useState<string | null>(null);
  const [evaluationError, setEvaluationError] = useState("");
  const [savingIvs, setSavingIvs] = useState(false);
  const [savingIdentity, setSavingIdentity] = useState(false);
  const [savingCp, setSavingCp] = useState(false);
  const [lifecycleAction, setLifecycleAction] = useState<
    "evolution" | "screenshot" | null
  >(null);
  const recordOperation = useRef(false);
  const operation = useRef<"selecting" | "saving" | null>(null);
  const previewUrl = useRef<string | null>(null);
  const mounted = useRef(false);
  const ocrAbort = useRef<AbortController | null>(null);
  const loadVersion = useRef(0);
  const reviewHeading = useRef<HTMLHeadingElement>(null);
  const selectButton = useRef<HTMLButtonElement>(null);

  function releasePreview() {
    if (previewUrl.current) URL.revokeObjectURL(previewUrl.current);
    previewUrl.current = null;
    setPreview(null);
  }

  async function loadInventory(retainDraftOnFailure = false) {
    const version = ++loadVersion.current;
    setLoadError("");
    try {
      const next = await native.snapshot();
      if (mounted.current && version === loadVersion.current) {
        setSnapshot(next);
        setSelectedId((current) =>
          next.records.some((record) => record.id === current) ? current : null,
        );
      }
    } catch (failure) {
      if (retainDraftOnFailure) throw failure;
      if (mounted.current && version === loadVersion.current)
        setLoadError(
          nativeErrorMessage(failure, "Unable to load the local inventory."),
        );
    }
  }

  useEffect(() => {
    mounted.current = true;
    void loadInventory();
    return () => {
      mounted.current = false;
      ocrAbort.current?.abort();
      if (previewUrl.current) URL.revokeObjectURL(previewUrl.current);
      previewUrl.current = null;
    };
  }, []);

  useEffect(() => {
    if (preview) reviewHeading.current?.focus();
  }, [preview]);

  useEffect(() => {
    if (phase === "idle" && status) selectButton.current?.focus();
  }, [phase, status]);
  async function selectScreenshot() {
    if (operation.current) return;
    operation.current = "selecting";
    setPhase("selecting");
    releasePreview();
    setFields(emptyReview);
    setEdited({});
    setError("");
    setStatus("");
    try {
      const image = await native.selectScreenshot();
      if (!mounted.current) return;
      if (image) {
        const url = URL.createObjectURL(
          new Blob([new Uint8Array(image.png)], { type: "image/png" }),
        );
        previewUrl.current = url;
        setPreview({
          url,
          width: image.width,
          height: image.height,
          extraction: image.extraction,
        });
        setPhase("extracting");
        const controller = new AbortController();
        ocrAbort.current = controller;
        const extraction = await extractText(image, controller.signal);
        if (!mounted.current || controller.signal.aborted) return;
        setPreview({
          url,
          width: image.width,
          height: image.height,
          extraction,
        });
        setFields({
          displayName: extraction.displayName.value ?? "",
          cp: extraction.cp.value ?? "",
          attack: extraction.attack.value?.toString() ?? "",
          defense: extraction.defense.value?.toString() ?? "",
          hp: extraction.hp.value?.toString() ?? "",
        });
        setEdited({});
      } else {
        setStatus("Screenshot selection cancelled. Nothing was saved.");
      }
    } catch (failure) {
      if (ocrAbort.current?.signal.aborted) {
        if (mounted.current) {
          releasePreview();
          setFields(emptyReview);
          setStatus("Extraction cancelled. Nothing was saved.");
        }
        return;
      }
      if (mounted.current)
        setError(
          nativeErrorMessage(
            failure,
            "Unable to open the selected screenshot.",
          ),
        );
    } finally {
      ocrAbort.current = null;
      operation.current = null;
      if (mounted.current) setPhase("idle");
    }
  }

  function cancelReview() {
    if (operation.current) return;
    releasePreview();
    setFields(emptyReview);
    setError("");
    setStatus("Review cancelled. Nothing was saved.");
  }

  async function confirmSave(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (operation.current || !preview) return;
    const input = validateReview(fields);
    if (typeof input === "string") {
      setError(input);
      return;
    }
    operation.current = "saving";
    setPhase("saving");
    setError("");
    setStatus("");
    try {
      const record = await native.confirmImport(input);
      if (!mounted.current) return;
      setSnapshot((current) =>
        current
          ? { ...current, records: [...current.records, record] }
          : current,
      );
      releasePreview();
      setFields(emptyReview);
      setStatus(
        `Saved ${record.displayName} to your local inventory. Awaiting evaluation.`,
      );
    } catch (failure) {
      if (mounted.current)
        setError(
          nativeErrorMessage(
            failure,
            "Unable to save. Your review values are retained; please try again.",
          ),
        );
    } finally {
      operation.current = null;
      if (mounted.current) setPhase("idle");
    }
  }

  function changeField(key: keyof ReviewFields, value: string) {
    setFields((current) => ({ ...current, [key]: value }));
    setEdited((current) => ({ ...current, [key]: true }));
  }
  async function evaluateSelected() {
    if (!selectedId || recordOperation.current) return;
    recordOperation.current = true;
    setEvaluating(selectedId);
    setEvaluationError("");
    ++loadVersion.current;
    try {
      const record = await native.evaluate(selectedId);
      if (mounted.current)
        setSnapshot((current) =>
          current
            ? {
                ...current,
                records: current.records.map((existing) =>
                  existing.id === record.id ? record : existing,
                ),
              }
            : current,
        );
    } catch (failure) {
      if (mounted.current)
        setEvaluationError(
          nativeErrorMessage(
            failure,
            "Unable to evaluate appraisal. Your existing data is retained; try again.",
          ),
        );
    } finally {
      recordOperation.current = false;
      if (mounted.current) setEvaluating(null);
    }
  }
  async function saveSelectedIvs(
    expected: AppraisalIvs | null,
    corrected: AppraisalIvs | null,
  ) {
    if (!selectedId || recordOperation.current)
      throw new Error("Inventory action already in progress");
    recordOperation.current = true;
    setSavingIvs(true);
    setEvaluationError("");
    ++loadVersion.current;
    try {
      const record = await native.correctIvs(selectedId, expected, corrected);
      if (mounted.current)
        setSnapshot((current) =>
          current
            ? {
                ...current,
                records: current.records.map((existing) =>
                  existing.id === record.id ? record : existing,
                ),
              }
            : current,
        );
    } finally {
      recordOperation.current = false;
      if (mounted.current) setSavingIvs(false);
    }
  }
  async function saveSelectedCp(
    expected: string | null,
    revision: string,
    cp: string | null,
  ) {
    if (!selectedId || recordOperation.current)
      throw new Error("Inventory action already in progress");
    recordOperation.current = true;
    setSavingCp(true);
    ++loadVersion.current;
    try {
      const record = await native.correctCp(selectedId, expected, revision, cp);
      if (mounted.current)
        setSnapshot((current) =>
          current
            ? {
                ...current,
                records: current.records.map((existing) =>
                  existing.id === record.id ? record : existing,
                ),
              }
            : current,
        );
    } finally {
      recordOperation.current = false;
      if (mounted.current) setSavingCp(false);
    }
  }
  async function saveSelectedIdentity(
    expected: ReferenceIdentity | null,
    identity: ReferenceIdentity | null,
  ) {
    if (!selectedId || recordOperation.current)
      throw new Error("Inventory action already in progress");
    recordOperation.current = true;
    setSavingIdentity(true);
    ++loadVersion.current;
    try {
      const record = await native.correctIdentity(
        selectedId,
        expected,
        identity,
      );
      if (mounted.current)
        setSnapshot((current) =>
          current
            ? {
                ...current,
                records: current.records.map((existing) =>
                  existing.id === record.id ? record : existing,
                ),
              }
            : current,
        );
    } finally {
      recordOperation.current = false;
      if (mounted.current) setSavingIdentity(false);
    }
  }
  async function commitLifecycle(action: () => Promise<InventoryRecord>) {
    if (recordOperation.current)
      throw new Error("Inventory action already in progress");
    recordOperation.current = true;
    ++loadVersion.current;
    try {
      const record = await action();
      if (mounted.current)
        setSnapshot((current) =>
          current
            ? {
                ...current,
                records: current.records.map((existing) =>
                  existing.id === record.id ? record : existing,
                ),
              }
            : current,
        );
    } finally {
      recordOperation.current = false;
    }
  }
  function evidence(key: keyof ReviewFields) {
    const field = preview?.extraction[key];
    if (!field) return null;
    if (phase === "extracting" && (key === "displayName" || key === "cp"))
      return <p id={`${key}-evidence`}>Reading locally…</p>;
    return (
      <p id={`${key}-evidence`}>
        {edited[key]
          ? field.value === null
            ? "Manually entered."
            : `Manually edited. Original suggestion: ${field.value}.`
          : field.state === "detected"
            ? `Detected suggestion: ${field.value}. Tentative confidence · ${field.method === "tesseract_lstm" ? `local OCR${field.engineConfidence === undefined ? "" : ` (engine score ${field.engineConfidence.toFixed(0)}/100)`}` : "appraisal bar geometry"}. Verify against the screenshot.`
            : field.state === "unavailable"
              ? "Unavailable: enter visible text manually."
              : "Unresolved: enter manually or leave unknown."}
      </p>
    );
  }
  const busy = phase !== "idle";

  return (
    <div className="inventory">
      <section aria-labelledby="import-title">
        <h2 id="import-title">Screenshot review</h2>
        <p>
          Select one local PNG or JPEG. The app reads the display name, visible
          CP and appraisal IVs locally, then fills the fields it can detect.
          Review and correct every suggestion before saving.
        </p>
        <p>
          The screenshot stays local and is released after review. Only the
          fields you explicitly confirm are saved.
        </p>
        <p>
          Review confirms the Pokémon’s data only. Confirmed records await
          evaluation. Run appraisal separately below; no tags are applied
          automatically.
        </p>
        <button
          ref={selectButton}
          type="button"
          onClick={() => void selectScreenshot()}
          disabled={busy || !snapshot || lifecycleAction !== null}
        >
          {phase === "selecting"
            ? "Selecting screenshot…"
            : preview
              ? "Replace screenshot"
              : "Select screenshot"}
        </button>
        <p role="status" aria-live="polite">
          {phase === "extracting"
            ? "Reading the screenshot locally…"
            : phase === "saving"
              ? "Saving your confirmed record…"
              : status}
        </p>
        {phase === "extracting" && (
          <button type="button" onClick={() => ocrAbort.current?.abort()}>
            Cancel extraction
          </button>
        )}
        {error && (
          <p role="alert" id="review-error">
            {error}
          </p>
        )}
        {preview && (
          <div className="review-layout" aria-busy={busy}>
            <figure>
              <img
                src={preview.url}
                alt="Selected Pokémon screenshot for manual review"
              />
              <figcaption>
                Local preview · {preview.width} × {preview.height}
              </figcaption>
            </figure>
            <form onSubmit={(event) => void confirmSave(event)} noValidate>
              <h3 ref={reviewHeading} tabIndex={-1}>
                Review and correct details
              </h3>
              <p>
                Tentative confidence means a geometry rule matched, not a
                measured probability of correctness. OCR scores are engine
                estimates, not measured accuracy. Nothing is saved until you
                confirm.
              </p>
              <ul aria-label="Extraction notices">
                {preview.extraction.issues
                  .filter((issue) => issue.code !== "text_unavailable")
                  .map((issue) => (
                    <li key={issue.code}>{issue.message}</li>
                  ))}
              </ul>
              <fieldset disabled={busy}>
                <legend>Supported fields</legend>
                <label htmlFor="display-name">Display name (required)</label>
                <input
                  id="display-name"
                  autoComplete="off"
                  value={fields.displayName}
                  onChange={(event) =>
                    changeField("displayName", event.target.value)
                  }
                  aria-describedby={`displayName-evidence${error ? " review-error" : ""}`}
                  required
                />
                {evidence("displayName")}
                <label htmlFor="cp">CP (optional)</label>
                <input
                  id="cp"
                  type="text"
                  inputMode="numeric"
                  autoComplete="off"
                  value={fields.cp}
                  onChange={(event) => changeField("cp", event.target.value)}
                  aria-describedby="cp-help cp-evidence"
                />
                {evidence("cp")}
                <p id="cp-help">
                  Leave empty if unknown. Whole number, 1–9223372036854775807.
                </p>
                <fieldset>
                  <legend>Appraisal IVs (optional group)</legend>
                  <p id="ivs-help">
                    Leave all three empty if unknown, or supply all three from 0
                    to 15. HP is the appraisal IV, not observed HP.
                  </p>
                  {(["attack", "defense", "hp"] as const).map((key) => (
                    <div key={key}>
                      <label htmlFor={`iv-${key}`}>
                        {key === "hp"
                          ? "HP IV"
                          : `${key === "attack" ? "Attack" : "Defense"} IV`}
                      </label>
                      <input
                        id={`iv-${key}`}
                        type="text"
                        inputMode="numeric"
                        value={fields[key]}
                        onChange={(event) =>
                          changeField(key, event.target.value)
                        }
                        aria-describedby={`ivs-help ${key}-evidence`}
                      />
                      {evidence(key)}
                    </div>
                  ))}
                </fieldset>
                <p>
                  Confirm these data to save a new record awaiting evaluation.
                </p>
                <button type="submit">Confirm and save</button>{" "}
                <button type="button" onClick={cancelReview}>
                  Cancel review
                </button>
              </fieldset>
            </form>
          </div>
        )}
      </section>
      <section aria-labelledby="records-title">
        <h2 id="records-title">Local collection</h2>
        <p>
          Existing tags are preserved and do not indicate a completed
          evaluation.
        </p>
        {loadError ? (
          <div role="alert">
            <p>{loadError}</p>
            <button type="button" onClick={() => void loadInventory()}>
              Retry loading inventory
            </button>
          </div>
        ) : !snapshot ? (
          <p role="status">Loading local inventory…</p>
        ) : snapshot.records.length === 0 ? (
          <p>No Pokémon saved yet. Confirm a screenshot review to add one.</p>
        ) : (
          <ul className="inventory-list">
            {snapshot.records.map((record) => (
              <li key={record.id}>
                <h3>{record.displayName}</h3>
                <p>Evaluation: {evaluationLabels[record.evaluation.state]}</p>
                <p>Appraisal only · Strategic analysis not performed</p>
                <p>
                  Species: {record.reference.identity?.species ?? "Unknown"}
                </p>
                <p>CP: {record.cp ?? "Unknown"}</p>
                <p>
                  Appraisal IVs:{" "}
                  {record.ivs
                    ? `${record.ivs.attack} / ${record.ivs.defense} / ${record.ivs.hp} (attack / defense / HP)`
                    : "Unknown"}
                </p>
                <p>
                  Tags:{" "}
                  {record.tags.length
                    ? record.tags.map((tag) => tag.name).join(", ")
                    : "None"}
                </p>
                <button
                  type="button"
                  aria-expanded={selectedId === record.id}
                  aria-controls={
                    selectedId === record.id ? "appraisal-panel" : undefined
                  }
                  disabled={
                    lifecycleAction !== null ||
                    evaluating !== null ||
                    savingIvs ||
                    savingIdentity ||
                    savingCp
                  }
                  onClick={() => {
                    setSelectedId(record.id);
                    setEvaluationError("");
                  }}
                >
                  Review appraisal for {record.displayName}
                </button>
              </li>
            ))}
          </ul>
        )}
        {!loadError &&
          snapshot?.records.find((record) => record.id === selectedId) && (
            <div id="appraisal-panel">
              <AppraisalPanel
                key={selectedId}
                record={
                  snapshot.records.find((record) => record.id === selectedId)!
                }
                busy={evaluating !== null}
                disabled={
                  savingIdentity || savingCp || lifecycleAction !== null
                }
                error={evaluationError}
                onEvaluate={() => void evaluateSelected()}
                onReload={() => void loadInventory()}
                savingIvs={savingIvs}
                onSaveIvs={saveSelectedIvs}
              />
              <CpCorrectionPanel
                key={`cp-${selectedId}`}
                record={
                  snapshot.records.find((record) => record.id === selectedId)!
                }
                busy={
                  evaluating !== null ||
                  savingIvs ||
                  savingIdentity ||
                  lifecycleAction !== null
                }
                saving={savingCp}
                onSave={saveSelectedCp}
                onReload={() => loadInventory(true)}
              />
              <EvolutionPanel
                key={"evolution-" + selectedId}
                record={
                  snapshot.records.find((record) => record.id === selectedId)!
                }
                busy={
                  busy ||
                  preview !== null ||
                  evaluating !== null ||
                  savingIvs ||
                  savingIdentity ||
                  savingCp ||
                  lifecycleAction === "screenshot"
                }
                onCommit={commitLifecycle}
                onActiveChange={(active) =>
                  setLifecycleAction(active ? "evolution" : null)
                }
                onReload={() => loadInventory(true)}
              />
              <ReobservationPanel
                key={"screenshot-" + selectedId}
                record={
                  snapshot.records.find((record) => record.id === selectedId)!
                }
                busy={
                  busy ||
                  preview !== null ||
                  evaluating !== null ||
                  savingIvs ||
                  savingIdentity ||
                  savingCp ||
                  lifecycleAction === "evolution"
                }
                onCommit={commitLifecycle}
                onActiveChange={(active) =>
                  setLifecycleAction(active ? "screenshot" : null)
                }
                onReload={() => loadInventory(true)}
              />
              <HistoryPanel
                key={"history-" + selectedId}
                record={
                  snapshot.records.find((record) => record.id === selectedId)!
                }
              />
              <ReferencePanel
                key={`reference-${selectedId}`}
                reference={
                  snapshot.records.find((record) => record.id === selectedId)!
                    .reference
                }
                busy={
                  evaluating !== null ||
                  savingIvs ||
                  savingCp ||
                  lifecycleAction !== null
                }
                saving={savingIdentity}
                onSave={saveSelectedIdentity}
                onReload={() => void loadInventory()}
              />
            </div>
          )}
      </section>
    </div>
  );
}
