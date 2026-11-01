import { useEffect, useRef, useState } from "react";
import { batchNative, type BatchSelection } from "./batch-native";
import {
  native,
  nativeErrorMessage,
  type ScreenshotExtraction,
} from "./native";
import { extractText } from "./ocr";
import { emptyReview, validateReview, type ReviewFields } from "./review";

type QueueState =
  | "Queued"
  | "Needs review"
  | "Missing optional information"
  | "Ready to confirm"
  | "Extraction failed"
  | "Grouped"
  | "Saved"
  | "Skipped";
interface Item {
  requestId: string;
  state: QueueState;
  fields: ReviewFields;
  extraction: ScreenshotExtraction | null;
  edited: Partial<Record<keyof ReviewFields, boolean>>;
  group?: number[];
  groupParent?: number;
  conflicts?: (keyof ReviewFields)[];
}
const labels: Record<keyof ReviewFields, string> = {
  displayName: "Display name",
  cp: "CP",
  attack: "Attack IV",
  defense: "Defense IV",
  hp: "HP IV",
};

function reviewState(fields: ReviewFields): QueueState {
  const validated = validateReview(fields);
  if (typeof validated === "string") return "Needs review";
  return validated.cp === null || validated.ivs === null
    ? "Missing optional information"
    : "Ready to confirm";
}

export function BatchImport({
  disabled = false,
  onSaved,
  onActiveChange,
}: {
  disabled?: boolean;
  onSaved: () => void | Promise<void>;
  onActiveChange?: (active: boolean) => void;
}) {
  const [session, setSession] = useState<BatchSelection | null>(null);
  const [items, setItems] = useState<Item[]>([]);
  const [selected, setSelected] = useState(0);
  const [busy, setBusy] = useState(false);
  const [preview, setPreview] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const [confirmed, setConfirmed] = useState(false);
  const [groupSelection, setGroupSelection] = useState<number[]>([]);
  const alive = useRef(false);
  const operation = useRef(false);
  const abort = useRef<AbortController | null>(null);
  const activeSession = useRef<BatchSelection | null>(null);
  const url = useRef<string | null>(null);
  const activeChange = useRef(onActiveChange);
  activeChange.current = onActiveChange;

  function clearPreview() {
    if (url.current) URL.revokeObjectURL(url.current);
    url.current = null;
    setPreview(null);
  }
  async function release(selection: BatchSelection | null) {
    if (selection)
      await batchNative.release(selection.sessionId).catch(() => undefined);
  }
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
      abort.current?.abort();
      if (url.current) URL.revokeObjectURL(url.current);
      void release(activeSession.current);
      activeChange.current?.(false);
    };
  }, []);

  async function open() {
    if (operation.current || disabled) return;
    operation.current = true;
    setBusy(true);
    setError("");
    setStatus("");
    try {
      const selection = await batchNative.select();
      if (!alive.current) {
        await release(selection);
        return;
      }
      if (!selection) {
        setStatus("Batch selection cancelled. Nothing was saved.");
        return;
      }
      await release(activeSession.current);
      activeSession.current = selection;
      setSession(selection);
      setItems(
        Array.from({ length: selection.count }, () => ({
          requestId: crypto.randomUUID(),
          state: "Queued",
          fields: { ...emptyReview },
          extraction: null,
          edited: {},
        })),
      );
      setSelected(0);
      setGroupSelection([]);
      setConfirmed(false);
      clearPreview();
      activeChange.current?.(true);
      setStatus(
        `${selection.count} screenshots selected. Extract the queue locally, then review each individual.`,
      );
    } catch (failure) {
      if (alive.current)
        setError(nativeErrorMessage(failure, "Unable to select screenshots."));
    } finally {
      operation.current = false;
      if (alive.current) setBusy(false);
    }
  }

  async function extractQueue() {
    if (!session || operation.current) return;
    operation.current = true;
    setBusy(true);
    setError("");
    clearPreview();
    const controller = new AbortController();
    abort.current = controller;
    const currentSession = session;
    let completed = 0;
    try {
      for (let index = 0; index < items.length; index++) {
        if (items[index].state !== "Queued") continue;
        if (
          !alive.current ||
          controller.signal.aborted ||
          activeSession.current !== currentSession
        )
          break;
        setStatus(
          `Extracting screenshot ${index + 1} of ${items.length} locally…`,
        );
        try {
          // The image/worker live for this iteration only. Keep structured facts,
          // never preview bytes or URLs, while processing the rest of the queue.
          const image = await batchNative.preview(
            currentSession.sessionId,
            index,
          );
          if (
            controller.signal.aborted ||
            activeSession.current !== currentSession
          )
            break;
          const extraction = await extractText(image, controller.signal);
          if (
            !alive.current ||
            controller.signal.aborted ||
            activeSession.current !== currentSession
          )
            break;
          const fields: ReviewFields = {
            displayName: extraction.displayName.value ?? "",
            cp: extraction.cp.value ?? "",
            attack: extraction.attack.value?.toString() ?? "",
            defense: extraction.defense.value?.toString() ?? "",
            hp: extraction.hp.value?.toString() ?? "",
          };
          setItems((current) =>
            current.map((item, i) =>
              i === index
                ? { ...item, fields, extraction, state: reviewState(fields) }
                : item,
            ),
          );
          completed++;
        } catch {
          if (controller.signal.aborted) break;
          if (alive.current && activeSession.current === currentSession)
            setItems((current) =>
              current.map((item, i) =>
                i === index ? { ...item, state: "Extraction failed" } : item,
              ),
            );
        }
      }
      if (
        alive.current &&
        !controller.signal.aborted &&
        activeSession.current === currentSession
      )
        setStatus(
          `Local extraction finished: ${completed} screenshots processed. Review uncertain fields and failed items before confirming.`,
        );
    } finally {
      abort.current = null;
      operation.current = false;
      if (alive.current) setBusy(false);
    }
  }

  function groupScreenshots() {
    if (operation.current || groupSelection.length < 2) return;
    const chosen = [...groupSelection].sort((a, b) => a - b);
    const target = chosen[0];
    const fields = { ...emptyReview };
    const conflicts: (keyof ReviewFields)[] = [];
    for (const key of Object.keys(labels) as (keyof ReviewFields)[]) {
      const values = [
        ...new Set(
          chosen.map((index) => items[index].fields[key]).filter(Boolean),
        ),
      ];
      if (values.length === 1) fields[key] = values[0];
      if (values.length > 1) conflicts.push(key);
    }
    if (
      conflicts.some(
        (key) => key === "attack" || key === "defense" || key === "hp",
      )
    ) {
      fields.attack = fields.defense = fields.hp = "";
    }
    setItems((current) =>
      current.map((item, index) =>
        index === target
          ? {
              ...item,
              fields,
              extraction: null,
              edited: {},
              group: chosen,
              conflicts,
              state: reviewState(fields),
            }
          : chosen.includes(index)
            ? { ...item, groupParent: target, state: "Grouped" }
            : item,
      ),
    );
    setSelected(target);
    setGroupSelection([]);
    setConfirmed(false);
    clearPreview();
    setStatus(
      `Screenshots ${chosen.map((index) => index + 1).join(", ")} grouped for one new individual. Review the combined facts; conflicting values were cleared to unknown.`,
    );
  }

  async function inspect(index: number) {
    if (!session || operation.current) return;
    operation.current = true;
    setBusy(true);
    setSelected(index);
    setConfirmed(false);
    setError("");
    clearPreview();
    const currentSession = session;
    const controller = new AbortController();
    abort.current = controller;
    try {
      const image = await batchNative.preview(session.sessionId, index);
      if (!alive.current || controller.signal.aborted) return;
      const imageUrl = URL.createObjectURL(
        new Blob([new Uint8Array(image.png)], { type: "image/png" }),
      );
      url.current = imageUrl;
      setPreview(imageUrl);
      // Existing drafts survive navigation; do not overwrite manual corrections.
      if (items[index].extraction || items[index].group) return;
      const extraction = await extractText(image, controller.signal);
      if (
        !alive.current ||
        controller.signal.aborted ||
        activeSession.current !== currentSession
      )
        return;
      const fields: ReviewFields = {
        displayName: extraction.displayName.value ?? "",
        cp: extraction.cp.value ?? "",
        attack: extraction.attack.value?.toString() ?? "",
        defense: extraction.defense.value?.toString() ?? "",
        hp: extraction.hp.value?.toString() ?? "",
      };
      setItems((current) =>
        current.map((item, i) =>
          i === index
            ? { ...item, fields, extraction, state: reviewState(fields) }
            : item,
        ),
      );
    } catch (failure) {
      if (alive.current && !controller.signal.aborted) {
        setItems((current) =>
          current.map((item, i) =>
            i === index ? { ...item, state: "Extraction failed" } : item,
          ),
        );
        setError(
          nativeErrorMessage(
            failure,
            "Extraction failed for this screenshot. Retry it, skip it, or enter known details manually.",
          ),
        );
      }
    } finally {
      abort.current = null;
      operation.current = false;
      if (alive.current) setBusy(false);
    }
  }

  function change(key: keyof ReviewFields, value: string) {
    setConfirmed(false);
    setItems((current) =>
      current.map((item, i) => {
        if (i !== selected) return item;
        const fields = { ...item.fields, [key]: value };
        return {
          ...item,
          fields,
          edited: { ...item.edited, [key]: true },
          state: reviewState(fields),
        };
      }),
    );
  }

  async function save() {
    if (operation.current || !confirmed) return;
    const input = validateReview(items[selected].fields);
    if (typeof input === "string") {
      setError(input);
      return;
    }
    operation.current = true;
    setBusy(true);
    setError("");
    try {
      await native.confirmBatchImport(items[selected].requestId, input);
      if (!alive.current) return;
      setItems((current) =>
        current.map((item, i) =>
          i === selected || items[selected].group?.includes(i)
            ? { ...item, state: "Saved" }
            : item,
        ),
      );
      setConfirmed(false);
      clearPreview();
      setStatus(
        `Screenshot ${selected + 1} saved as a new individual. Remaining screenshots are unchanged.`,
      );
      await onSaved();
    } catch (failure) {
      if (alive.current)
        setError(
          nativeErrorMessage(
            failure,
            "Unable to save this reviewed individual. Review the local inventory before retrying if the result is uncertain.",
          ),
        );
    } finally {
      operation.current = false;
      if (alive.current) setBusy(false);
    }
  }

  async function cancel() {
    if (operation.current && !abort.current) return;
    abort.current?.abort();
    const previous = activeSession.current;
    activeSession.current = null;
    setSession(null);
    setItems([]);
    setGroupSelection([]);
    setConfirmed(false);
    clearPreview();
    activeChange.current?.(false);
    setStatus(
      "Batch closed. Confirmed individuals are retained; unsaved drafts were discarded. Original screenshots are unchanged.",
    );
    await release(previous);
  }

  const item = items[selected];
  const editable =
    item && !["Saved", "Skipped", "Queued", "Grouped"].includes(item.state);
  return (
    <section aria-labelledby="batch-title">
      <h2 id="batch-title">Batch screenshot import</h2>
      <p>
        Select up to 500 PNG/JPEG screenshots. Each explicitly confirmed review
        creates a new UUID. Matching images are never automatically merged. Use
        the selected individual’s screenshot update to add observations to an
        existing UUID.
      </p>
      <button
        type="button"
        disabled={disabled || busy || session !== null}
        onClick={() => void open()}
      >
        Select screenshot batch
      </button>
      {status && <p role="status">{status}</p>}
      {error && <p role="alert">{error}</p>}
      {session && (
        <>
          <p>
            {items.filter((entry) => entry.state === "Saved").length} saved /{" "}
            {items.length} selected. Images are decoded one at a time; names and
            file paths are not saved.
          </p>
          <button
            type="button"
            disabled={busy || !items.some((entry) => entry.state === "Queued")}
            onClick={() => void extractQueue()}
          >
            Extract queued screenshots
          </button>{" "}
          <button
            type="button"
            disabled={busy && abort.current === null}
            onClick={() => void cancel()}
          >
            Close batch / cancel extraction
          </button>
          <ol aria-label="Batch review queue">
            {items.map((entry, index) => (
              <li key={index}>
                {!entry.group &&
                  entry.groupParent === undefined &&
                  !["Saved", "Skipped", "Queued"].includes(entry.state) && (
                    <label>
                      <input
                        type="checkbox"
                        disabled={busy}
                        checked={groupSelection.includes(index)}
                        onChange={(event) =>
                          setGroupSelection((current) =>
                            event.target.checked
                              ? [...current, index]
                              : current.filter((value) => value !== index),
                          )
                        }
                      />
                      Group screenshot {index + 1}
                    </label>
                  )}
                <button
                  type="button"
                  disabled={
                    busy || entry.state === "Saved" || entry.state === "Skipped"
                  }
                  aria-current={selected === index ? "step" : undefined}
                  onClick={() => void inspect(index)}
                >
                  Review screenshot {index + 1}
                </button>{" "}
                <span>{entry.state}</span>
                {entry.fields.displayName
                  ? ` · ${entry.fields.displayName}`
                  : ""}
              </li>
            ))}
          </ol>
          <button
            type="button"
            disabled={busy || groupSelection.length < 2}
            onClick={groupScreenshots}
          >
            Group selected screenshots as one individual
          </button>
          <p>
            Only group screenshots you explicitly recognize as the same
            individual. Grouping creates one new UUID after review; it never
            changes an existing individual.
          </p>
          {item && (
            <div aria-busy={busy}>
              <h3>Screenshot {selected + 1}</h3>
              {preview && (
                <img
                  src={preview}
                  alt={`Screenshot ${selected + 1} temporary review preview`}
                  style={{ maxWidth: "100%", maxHeight: "36rem" }}
                />
              )}
              {busy && <p role="status">Processing locally…</p>}
              {editable && (
                <fieldset disabled={busy}>
                  <legend>Review extracted or manually entered facts</legend>
                  <p>
                    OCR and bar detections are tentative suggestions, not
                    measured accuracy. Empty optional values remain unknown.
                    Shiny status, moves and other unsupported fields are not
                    inferred.
                  </p>
                  {item.group && (
                    <>
                      <p>
                        Grouped screenshots:{" "}
                        {item.group.map((index) => index + 1).join(", ")}.
                        Confirm that every screenshot belongs to this same new
                        individual.
                      </p>
                      {item.conflicts?.length ? (
                        <p role="alert">
                          Conflicting{" "}
                          {item.conflicts.map((key) => labels[key]).join(", ")}{" "}
                          values were cleared. Enter confirmed values or leave
                          optional values unknown.
                        </p>
                      ) : null}
                    </>
                  )}
                  {(Object.keys(labels) as (keyof ReviewFields)[]).map(
                    (key) => (
                      <div key={key}>
                        <label htmlFor={`batch-${key}`}>
                          {labels[key]}
                          {key === "displayName"
                            ? " (required)"
                            : " (optional)"}
                        </label>
                        <input
                          id={`batch-${key}`}
                          inputMode={key === "displayName" ? "text" : "numeric"}
                          value={item.fields[key]}
                          onChange={(event) => change(key, event.target.value)}
                          aria-describedby={`batch-${key}-evidence`}
                        />
                        <p id={`batch-${key}-evidence`}>
                          {item.edited[key]
                            ? "Manually entered"
                            : item.group && item.fields[key]
                              ? "Grouped review suggestion · explicitly confirm"
                              : item.extraction?.[key].value !== null &&
                                  item.extraction?.[key].value !== undefined
                                ? `Extracted · ${item.extraction[key].method} · ${item.extraction[key].confidence}`
                                : "Unknown"}
                        </p>
                      </div>
                    ),
                  )}
                  <p>
                    Provide all three appraisal IVs (0–15), or leave all three
                    empty.
                  </p>
                  <label>
                    <input
                      type="checkbox"
                      checked={confirmed}
                      onChange={(event) => setConfirmed(event.target.checked)}
                    />
                    {item.group
                      ? "I reviewed every grouped screenshot and confirm they show the same new individual"
                      : "I reviewed these facts and confirm this is a new individual"}
                  </label>
                  <button
                    type="button"
                    disabled={!confirmed}
                    onClick={() => void save()}
                  >
                    Confirm batch individual
                  </button>{" "}
                  <button
                    type="button"
                    onClick={() => {
                      setItems((current) =>
                        current.map((entry, index) =>
                          index === selected || item.group?.includes(index)
                            ? { ...entry, state: "Skipped" }
                            : entry,
                        ),
                      );
                      clearPreview();
                      setConfirmed(false);
                    }}
                  >
                    Skip screenshot
                  </button>
                </fieldset>
              )}
            </div>
          )}
        </>
      )}
    </section>
  );
}
