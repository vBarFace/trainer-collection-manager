import { useEffect, useRef, useState } from "react";
import type { FormEvent } from "react";
import {
  native,
  nativeErrorMessage,
  type InventorySnapshot,
  type ScreenshotExtraction,
} from "./native";
import { emptyReview, validateReview, type ReviewFields } from "./review";

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
  const [phase, setPhase] = useState<"idle" | "selecting" | "saving">("idle");
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const operation = useRef<"selecting" | "saving" | null>(null);
  const previewUrl = useRef<string | null>(null);
  const mounted = useRef(false);
  const loadVersion = useRef(0);
  const reviewHeading = useRef<HTMLHeadingElement>(null);
  const selectButton = useRef<HTMLButtonElement>(null);

  function releasePreview() {
    if (previewUrl.current) URL.revokeObjectURL(previewUrl.current);
    previewUrl.current = null;
    setPreview(null);
  }

  async function loadInventory() {
    const version = ++loadVersion.current;
    setLoadError("");
    try {
      const next = await native.snapshot();
      if (mounted.current && version === loadVersion.current) setSnapshot(next);
    } catch (failure) {
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
        const extraction = image.extraction;
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
      if (mounted.current)
        setError(
          nativeErrorMessage(
            failure,
            "Unable to open the selected screenshot.",
          ),
        );
    } finally {
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
  function evidence(key: keyof ReviewFields) {
    const field = preview?.extraction[key];
    if (!field) return null;
    return (
      <p id={`${key}-evidence`}>
        {edited[key]
          ? field.value === null
            ? "Manually entered."
            : `Manually edited. Original suggestion: ${field.value}.`
          : field.state === "detected"
            ? `Detected suggestion: ${field.value}. Tentative confidence · experimental appraisal bar geometry. Verify against the screenshot.`
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
          Select one local PNG or JPEG. Experimental local detection can suggest
          IVs from three clear appraisal bars. Review every suggestion and enter
          the display name and visible CP manually. Text recognition is
          unavailable.
        </p>
        <p>
          The screenshot stays local and is released after review. Only the
          fields you explicitly confirm are saved.
        </p>
        <p>
          Review confirms the Pokémon’s data only. Confirmed records await
          evaluation; evaluation and automatic tagging are not available yet.
        </p>
        <button
          ref={selectButton}
          type="button"
          onClick={() => void selectScreenshot()}
          disabled={busy || !snapshot}
        >
          {phase === "selecting"
            ? "Selecting screenshot…"
            : preview
              ? "Replace screenshot"
              : "Select screenshot"}
        </button>
        <p role="status" aria-live="polite">
          {phase === "saving" ? "Saving your confirmed record…" : status}
        </p>
        {error && (
          <p role="alert" id="review-error">
            {error}
          </p>
        )}
        {preview && (
          <div className="review-layout" aria-busy={phase === "saving"}>
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
                measured probability of correctness. Other graphics can resemble
                bars. Nothing is saved until you confirm.
              </p>
              <ul aria-label="Extraction notices">
                {preview.extraction.issues.map((issue) => (
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
        <h2 id="records-title">Awaiting evaluation</h2>
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
          <p>No Pokémon awaiting evaluation yet.</p>
        ) : (
          <ul className="inventory-list">
            {snapshot.records.map((record) => (
              <li key={record.id}>
                <h3>{record.displayName}</h3>
                <p>Evaluation: Pending</p>
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
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
