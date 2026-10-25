import { useEffect, useRef, useState } from "react";
import type { FormEvent } from "react";
import { native, nativeErrorMessage, type InventorySnapshot } from "./native";
import { emptyReview, validateReview, type ReviewFields } from "./review";

interface Preview {
  url: string;
  width: number;
  height: number;
}

export function Inventory() {
  const [snapshot, setSnapshot] = useState<InventorySnapshot | null>(null);
  const [loadError, setLoadError] = useState("");
  const [preview, setPreview] = useState<Preview | null>(null);
  const [fields, setFields] = useState<ReviewFields>(emptyReview);
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
        setPreview({ url, width: image.width, height: image.height });
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
      setStatus(`Saved ${record.displayName} to your local inventory.`);
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

  function changeField(key: keyof Omit<ReviewFields, "tagIds">, value: string) {
    setFields((current) => ({ ...current, [key]: value }));
  }
  const busy = phase !== "idle";

  return (
    <div className="inventory">
      <section aria-labelledby="import-title">
        <h2 id="import-title">Manual screenshot review</h2>
        <p>
          Select one local PNG or JPEG, then enter and review its details
          yourself. This workflow does not use OCR or extract any fields
          automatically.
        </p>
        <p>
          The screenshot stays local and is released after review. Only the
          fields you explicitly confirm are saved.
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
                Review details manually
              </h3>
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
                  aria-describedby={error ? "review-error" : undefined}
                  required
                />
                <label htmlFor="cp">CP (optional)</label>
                <input
                  id="cp"
                  type="text"
                  inputMode="numeric"
                  autoComplete="off"
                  value={fields.cp}
                  onChange={(event) => changeField("cp", event.target.value)}
                  aria-describedby="cp-help"
                />
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
                    <label key={key} htmlFor={`iv-${key}`}>
                      {key === "hp"
                        ? "HP IV"
                        : `${key === "attack" ? "Attack" : "Defense"} IV`}
                      <input
                        id={`iv-${key}`}
                        type="text"
                        inputMode="numeric"
                        value={fields[key]}
                        onChange={(event) =>
                          changeField(key, event.target.value)
                        }
                        aria-describedby="ivs-help"
                      />
                    </label>
                  ))}
                </fieldset>
                <fieldset>
                  <legend>Existing tags (optional)</legend>
                  {snapshot?.tags.map((tag) => (
                    <label key={tag.id}>
                      <input
                        type="checkbox"
                        checked={fields.tagIds.includes(tag.id)}
                        onChange={(event) =>
                          setFields((current) => ({
                            ...current,
                            tagIds: event.target.checked
                              ? [...current.tagIds, tag.id]
                              : current.tagIds.filter((id) => id !== tag.id),
                          }))
                        }
                      />
                      {tag.name}
                      {tag.scope === "local" ? " (custom)" : ""}
                    </label>
                  ))}
                  <p>
                    SAVE/SPECIAL is your choice to retain a Pokémon; it does not
                    identify shiny status.
                  </p>
                </fieldset>
                <p>
                  Confirm to save one new record with these details and tags.
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
        <h2 id="records-title">Saved inventory</h2>
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
          <p>No saved Pokémon yet.</p>
        ) : (
          <ul className="inventory-list">
            {snapshot.records.map((record) => (
              <li key={record.id}>
                <h3>{record.displayName}</h3>
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
