import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  native,
  nativeErrorMessage,
  type InventoryRecord,
  type EvolutionOptions,
  type HistoryEvent,
  type ScreenshotExtraction,
} from "./native";
import { extractText } from "./ocr";
import {
  emptyReview,
  validateCp,
  validateIvs,
  type ReviewFields,
} from "./review";

type Props = {
  record: InventoryRecord;
  busy: boolean;
  onCommit: (action: () => Promise<InventoryRecord>) => Promise<void>;
  onActiveChange: (active: boolean) => void;
  onReload: () => Promise<void>;
};
function conflict(failure: unknown) {
  return (
    typeof failure === "object" &&
    failure !== null &&
    "code" in failure &&
    (failure.code === "conflict" ||
      failure.code === "missing_pokemon" ||
      failure.code === "already_confirmed")
  );
}

export function EvolutionPanel({
  record,
  busy,
  onCommit,
  onActiveChange,
  onReload,
}: Props) {
  const [options, setOptions] = useState<EvolutionOptions | null>(null);
  const [target, setTarget] = useState("");
  const [confirmed, setConfirmed] = useState(false);
  const [working, setWorking] = useState(false);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const [stale, setStale] = useState(false);
  const pending = useRef(false);
  const alive = useRef(false);
  const request = useRef("");
  const button = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);
  async function begin() {
    if (busy || pending.current || options) return;
    pending.current = true;
    setWorking(true);
    setError("");
    setStatus("");
    setStale(false);
    onActiveChange(true);
    try {
      const next = await native.evolutionOptions(record.id);
      if (!alive.current) return;
      if (next.unavailableReason || !next.targets.length) {
        setStatus(
          next.unavailableReason ??
            "No confirmed normal evolution targets are available.",
        );
        onActiveChange(false);
      } else {
        request.current = crypto.randomUUID();
        setTarget("");
        setConfirmed(false);
        setOptions(next);
      }
    } catch (failure) {
      if (alive.current) {
        setError(
          nativeErrorMessage(failure, "Unable to load evolution targets."),
        );
        onActiveChange(false);
      }
    } finally {
      pending.current = false;
      if (alive.current) setWorking(false);
    }
  }
  function close(message: string) {
    setOptions(null);
    setTarget("");
    setConfirmed(false);
    setStale(false);
    onActiveChange(false);
    setStatus(message);
    button.current?.focus();
  }
  async function submit(event: FormEvent) {
    event.preventDefault();
    if (busy || pending.current || stale || !options || !target || !confirmed)
      return;
    pending.current = true;
    setWorking(true);
    setError("");
    try {
      await onCommit(() =>
        native.confirmEvolution(record.id, {
          requestId: request.current,
          expectedStrategicObservationRevision:
            options.expectedStrategicObservationRevision,
          datasetId: options.datasetId,
          datasetVersion: options.datasetVersion,
          targetFormId: target,
        }),
      );
      if (alive.current)
        close(
          "Evolution confirmed and saved. CP is now unknown; update it from a new screenshot when ready.",
        );
    } catch (failure) {
      if (alive.current) {
        setError(
          nativeErrorMessage(
            failure,
            "Unable to save evolution. The pending action is retained.",
          ),
        );
        setStale(conflict(failure));
      }
    } finally {
      pending.current = false;
      if (alive.current) setWorking(false);
    }
  }
  const chosen = options?.targets.find((item) => item.formId === target);
  return (
    <section aria-labelledby="evolution-title" aria-busy={working}>
      <h3 id="evolution-title">Confirmed evolution</h3>
      <p>Record a normal permanent evolution you performed in Pokémon GO.</p>
      <button
        ref={button}
        type="button"
        disabled={busy || working || options !== null}
        onClick={() => void begin()}
      >
        Evolve
      </button>
      {options && (
        <form onSubmit={(event) => void submit(event)}>
          <p>Pending action · Nothing is saved until confirmation.</p>
          <fieldset disabled={busy || working || stale}>
            <legend>Review evolution</legend>
            <label htmlFor="evolution-target">Evolution target</label>
            <select
              id="evolution-target"
              value={target}
              onChange={(event) => {
                setTarget(event.target.value);
                setConfirmed(false);
              }}
              required
            >
              <option value="">Select the actual target</option>
              {options.targets.map((item) => (
                <option key={item.formId} value={item.formId}>
                  {item.species} · {item.form}
                </option>
              ))}
            </select>
            {chosen && (
              <p>
                {record.reference.identity?.species} → {chosen.species} (
                {chosen.form}). Same individual UUID. CP becomes unknown. IVs,
                tags and display label are preserved.
              </p>
            )}
            <label>
              <input
                type="checkbox"
                checked={confirmed}
                onChange={(event) => setConfirmed(event.target.checked)}
              />
              I confirm this evolution occurred in Pokémon GO
            </label>
            <button type="submit" disabled={!target || !confirmed}>
              Confirm evolution
            </button>
          </fieldset>
          <button
            type="button"
            disabled={working}
            onClick={() => close("Evolution cancelled. Nothing was saved.")}
          >
            Cancel evolution
          </button>
        </form>
      )}
      {error && <p role="alert">{error}</p>}
      {stale && (
        <button
          type="button"
          disabled={working}
          onClick={() =>
            void onReload()
              .then(() =>
                close("Inventory reloaded. Start a new evolution review."),
              )
              .catch((failure: unknown) =>
                setError(
                  nativeErrorMessage(failure, "Unable to reload inventory."),
                ),
              )
          }
        >
          Reload current individual
        </button>
      )}
      {status && <p role="status">{status}</p>}
    </section>
  );
}

export function ReobservationPanel({
  record,
  busy,
  onCommit,
  onActiveChange,
  onReload,
}: Props) {
  const [draft, setDraft] = useState<InventoryRecord | null>(null);
  const [preview, setPreview] = useState<{
    url: string;
    extraction: ScreenshotExtraction;
  } | null>(null);
  const [fields, setFields] = useState<ReviewFields>(emptyReview);
  const [edited, setEdited] = useState<
    Partial<Record<keyof ReviewFields, boolean>>
  >({});
  const [applyCp, setApplyCp] = useState(false);
  const [applyIvs, setApplyIvs] = useState(false);
  const [species, setSpecies] = useState("");
  const [form, setForm] = useState("");
  const [confirmed, setConfirmed] = useState(false);
  const [phase, setPhase] = useState<
    "idle" | "selecting" | "extracting" | "saving"
  >("idle");
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const [stale, setStale] = useState(false);
  const pending = useRef(false);
  const alive = useRef(false);
  const url = useRef<string | null>(null);
  const abort = useRef<AbortController | null>(null);
  const request = useRef("");
  const button = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
      abort.current?.abort();
      if (url.current) URL.revokeObjectURL(url.current);
    };
  }, []);
  function release() {
    if (url.current) URL.revokeObjectURL(url.current);
    url.current = null;
    setPreview(null);
    setDraft(null);
    setFields(emptyReview);
  }
  function close(message: string) {
    release();
    setError("");
    setStale(false);
    onActiveChange(false);
    setStatus(message);
    button.current?.focus();
  }
  async function begin() {
    if (busy || pending.current || draft) return;
    const selected = structuredClone(record);
    pending.current = true;
    onActiveChange(true);
    setPhase("selecting");
    setError("");
    setStatus("");
    setStale(false);
    try {
      const image = await native.selectScreenshot();
      if (!alive.current) return;
      if (!image) {
        close("Screenshot selection cancelled. Nothing was saved.");
        return;
      }
      const nextUrl = URL.createObjectURL(
        new Blob([new Uint8Array(image.png)], { type: "image/png" }),
      );
      url.current = nextUrl;
      setDraft(selected);
      setPreview({ url: nextUrl, extraction: image.extraction });
      setPhase("extracting");
      const controller = new AbortController();
      abort.current = controller;
      const extraction = await extractText(image, controller.signal);
      if (!alive.current || controller.signal.aborted) return;
      setPreview({ url: nextUrl, extraction });
      setFields({
        displayName: extraction.displayName.value ?? "",
        cp: extraction.cp.value ?? "",
        attack: extraction.attack.value?.toString() ?? "",
        defense: extraction.defense.value?.toString() ?? "",
        hp: extraction.hp.value?.toString() ?? "",
      });
      setEdited({});
      setApplyCp(false);
      setApplyIvs(false);
      setConfirmed(false);
      setSpecies(selected.reference.identity?.species ?? "");
      setForm(selected.reference.identity?.form ?? "");
      request.current = crypto.randomUUID();
    } catch (failure) {
      if (alive.current) {
        if (abort.current?.signal.aborted)
          close("Extraction cancelled. Nothing was saved.");
        else {
          release();
          onActiveChange(false);
          setError(
            nativeErrorMessage(
              failure,
              "Unable to read screenshot. Try selecting it again.",
            ),
          );
        }
      }
    } finally {
      abort.current = null;
      pending.current = false;
      if (alive.current) setPhase("idle");
    }
  }
  const observedIdentity =
    species === "" && form === "" ? null : { species, form: form || null };
  const identityMismatch =
    !!draft &&
    JSON.stringify(observedIdentity) !==
      JSON.stringify(draft.reference.identity);
  function change(key: keyof ReviewFields, value: string) {
    setFields((current) => ({ ...current, [key]: value }));
    setEdited((current) => ({ ...current, [key]: true }));
    setConfirmed(false);
  }
  async function submit(event: FormEvent) {
    event.preventDefault();
    if (
      busy ||
      pending.current ||
      !draft ||
      !preview ||
      stale ||
      identityMismatch ||
      !confirmed ||
      (!applyCp && !applyIvs)
    )
      return;
    const cpError = applyCp ? validateCp(fields.cp) : null;
    const ivs = applyIvs ? validateIvs(fields) : null;
    if (
      cpError ||
      (applyCp && !fields.cp) ||
      typeof ivs === "string" ||
      (applyIvs && !ivs)
    ) {
      setError(
        cpError ??
          (typeof ivs === "string"
            ? ivs
            : "Enter a visible value for each selected update. Unselected or missing fields keep saved observations."),
      );
      return;
    }
    pending.current = true;
    setPhase("saving");
    setError("");
    try {
      await onCommit(() =>
        native.reobserve(draft.id, {
          requestId: request.current,
          expectedStrategicObservationRevision:
            draft.strategicObservationRevision,
          cp: applyCp
            ? {
                value: fields.cp,
                source:
                  edited.cp || preview.extraction.cp.value === null
                    ? "manual"
                    : "screenshot",
              }
            : null,
          ivs:
            applyIvs && ivs && typeof ivs !== "string"
              ? {
                  value: ivs,
                  source: (["attack", "defense", "hp"] as const).some(
                    (key) =>
                      edited[key] || preview.extraction[key].value === null,
                  )
                    ? "manual"
                    : "screenshot",
                }
              : null,
          observedIdentity,
          sameIndividualConfirmed: confirmed,
        }),
      );
      if (alive.current)
        close(
          "Observation updated for the same individual. No new Pokémon was created.",
        );
    } catch (failure) {
      if (alive.current) {
        setError(
          nativeErrorMessage(
            failure,
            "Unable to save observations. Your review is retained.",
          ),
        );
        setStale(conflict(failure));
      }
    } finally {
      pending.current = false;
      if (alive.current) setPhase("idle");
    }
  }
  function evidence(key: keyof ReviewFields) {
    const field = preview?.extraction[key];
    if (!field) return null;
    return (
      <p>
        {edited[key]
          ? "Manually entered or edited."
          : field.value === null
            ? "Missing or unresolved. Saved observation is kept unless you enter and select an update."
            : `Suggestion: ${field.value} · Tentative confidence · ${field.method}${field.engineConfidence === undefined ? "" : ` (engine score ${field.engineConfidence.toFixed(0)}/100)`}. Verify against the screenshot.`}
      </p>
    );
  }
  const working = phase !== "idle";
  const nameDiffers =
    preview?.extraction.displayName.value &&
    draft &&
    preview.extraction.displayName.value !== draft.displayName &&
    preview.extraction.displayName.value !== draft.reference.identity?.species;
  return (
    <section aria-labelledby="reobservation-title" aria-busy={working}>
      <h3 id="reobservation-title">Update existing observations</h3>
      <p>
        Review a new screenshot for this individual. Select each observation to
        replace; missing fields keep their saved values.
      </p>
      <button
        ref={button}
        type="button"
        disabled={busy || working || draft !== null}
        onClick={() => void begin()}
      >
        Update from Screenshot
      </button>
      {(phase === "selecting" || phase === "extracting") && (
        <p role="status">
          {phase === "selecting"
            ? "Selecting screenshot…"
            : "Reading the screenshot locally…"}
        </p>
      )}
      {phase === "extracting" && (
        <button type="button" onClick={() => abort.current?.abort()}>
          Cancel extraction
        </button>
      )}
      {draft && preview && (
        <div className="review-layout">
          <figure>
            <img
              src={preview.url}
              alt="Screenshot for the selected existing individual"
            />
            <figcaption>Temporary local preview</figcaption>
          </figure>
          <form onSubmit={(event) => void submit(event)} noValidate>
            <h4>Review observation changes for {draft.displayName}</h4>
            <p>
              Selected UUID: {draft.id}. Pending review · Nothing is saved yet.
            </p>
            <p>
              Detected display label:{" "}
              {preview.extraction.displayName.value ?? "Unresolved"}. Display
              labels may be nicknames; extraction does not establish species.
            </p>
            {evidence("displayName")}
            {nameDiffers && (
              <p role="alert">
                The screenshot label differs from the selected individual.
                Review its identity and species before continuing.
              </p>
            )}
            {preview.extraction.issues.map((issue) => (
              <p key={issue.code}>{issue.message}</p>
            ))}
            <fieldset disabled={busy || working || stale}>
              <legend>Identity review</legend>
              <p>
                Saved species: {draft.reference.identity?.species ?? "Unknown"}{" "}
                · form: {draft.reference.identity?.form ?? "Unspecified"}
              </p>
              <label htmlFor="observed-species">Observed species</label>
              <input
                id="observed-species"
                value={species}
                onChange={(event) => {
                  setSpecies(event.target.value);
                  setConfirmed(false);
                }}
              />
              <label htmlFor="observed-form">Observed form</label>
              <input
                id="observed-form"
                value={form}
                onChange={(event) => {
                  setForm(event.target.value);
                  setConfirmed(false);
                }}
              />
              {identityMismatch && (
                <p role="alert">
                  Species/form mismatch. This update cannot change species.
                  Cancel and use confirmed evolution or correct the saved
                  identity separately.
                </p>
              )}
              <p>
                Leave identity fields empty only when the saved identity is
                unknown.
              </p>
              <label>
                <input
                  type="checkbox"
                  checked={applyCp}
                  onChange={(event) => {
                    setApplyCp(event.target.checked);
                    setConfirmed(false);
                  }}
                />
                Update CP
              </label>
              <p>
                Saved CP: {draft.cp ?? "Unknown"} →{" "}
                {applyCp ? fields.cp || "Enter a value" : "Keep saved CP"}
              </p>
              <label htmlFor="observed-cp">Observed CP</label>
              <input
                id="observed-cp"
                inputMode="numeric"
                value={fields.cp}
                onChange={(event) => change("cp", event.target.value)}
              />
              {evidence("cp")}
              <label>
                <input
                  type="checkbox"
                  checked={applyIvs}
                  onChange={(event) => {
                    setApplyIvs(event.target.checked);
                    setConfirmed(false);
                  }}
                />
                Update appraisal IVs
              </label>
              <p>
                Saved appraisal IVs:{" "}
                {draft.ivs
                  ? `${draft.ivs.attack}/${draft.ivs.defense}/${draft.ivs.hp}`
                  : "Unknown"}{" "}
                →{" "}
                {applyIvs
                  ? `${fields.attack || "?"}/${fields.defense || "?"}/${fields.hp || "?"}`
                  : "Keep saved IVs"}
              </p>
              {(["attack", "defense", "hp"] as const).map((key) => (
                <div key={key}>
                  <label htmlFor={`observed-${key}`}>
                    Observed {key === "hp" ? "HP" : key} IV
                  </label>
                  <input
                    id={`observed-${key}`}
                    inputMode="numeric"
                    value={fields[key]}
                    onChange={(event) => change(key, event.target.value)}
                  />
                  {evidence(key)}
                </div>
              ))}
              <label>
                <input
                  type="checkbox"
                  checked={confirmed}
                  onChange={(event) => setConfirmed(event.target.checked)}
                />
                I verified this is the selected individual and its current
                species/form
              </label>
              <button
                type="submit"
                disabled={
                  !confirmed || identityMismatch || (!applyCp && !applyIvs)
                }
              >
                Confirm observation update
              </button>
            </fieldset>
            <button
              type="button"
              disabled={working}
              onClick={() =>
                close("Screenshot update cancelled. Nothing was saved.")
              }
            >
              Cancel screenshot update
            </button>
          </form>
        </div>
      )}
      {error && <p role="alert">{error}</p>}
      {stale && (
        <button
          type="button"
          disabled={working}
          onClick={() =>
            void onReload()
              .then(() =>
                close(
                  "Inventory reloaded. Select a new screenshot to continue.",
                ),
              )
              .catch((failure: unknown) =>
                setError(
                  nativeErrorMessage(failure, "Unable to reload inventory."),
                ),
              )
          }
        >
          Reload current individual
        </button>
      )}
      {status && <p role="status">{status}</p>}
    </section>
  );
}

export function HistoryPanel({ record }: { record: InventoryRecord }) {
  const [events, setEvents] = useState<HistoryEvent[] | null>(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const alive = useRef(false);
  const pending = useRef(false);
  const version = useRef(0);
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);
  useEffect(() => {
    ++version.current;
    setEvents(null);
  }, [record]);
  async function load() {
    if (pending.current) return;
    pending.current = true;
    const requestVersion = version.current;
    setLoading(true);
    setError("");
    try {
      const next = await native.history(record.id);
      if (alive.current && requestVersion === version.current) setEvents(next);
    } catch (failure) {
      if (alive.current && requestVersion === version.current)
        setError(
          nativeErrorMessage(failure, "Unable to load confirmed history."),
        );
    } finally {
      pending.current = false;
      if (alive.current) setLoading(false);
    }
  }
  return (
    <section aria-labelledby="history-title">
      <h3 id="history-title">Confirmed history</h3>
      <button type="button" disabled={loading} onClick={() => void load()}>
        Load confirmed history
      </button>
      {error && <p role="alert">{error}</p>}
      {events?.length === 0 && <p>No confirmed lifecycle events yet.</p>}
      {events && (
        <ol>
          {events.map((event) => (
            <li key={event.requestId}>
              <p>
                {event.kind === "evolution"
                  ? "Confirmed evolution"
                  : "Updated observation"}{" "}
                · {event.confirmedAt} UTC
              </p>
              <p>
                Species: {event.before.identity?.species ?? "Unknown"} →{" "}
                {event.after.identity?.species ?? "Unknown"}. CP:{" "}
                {event.before.cp ?? "Unknown"} → {event.after.cp ?? "Unknown"}
                {event.cpSource ? ` (${event.cpSource}, user confirmed)` : ""}.
              </p>
              <p>
                Reference: {event.before.speciesId ?? "Unknown"} /{" "}
                {event.before.formId ?? "Unknown"} →{" "}
                {event.after.speciesId ?? "Unknown"} /{" "}
                {event.after.formId ?? "Unknown"} · {event.datasetId} /{" "}
                {event.datasetVersion}
              </p>
              <p>
                IVs:{" "}
                {event.before.ivs
                  ? `${event.before.ivs.attack}/${event.before.ivs.defense}/${event.before.ivs.hp}`
                  : "Unknown"}{" "}
                →{" "}
                {event.after.ivs
                  ? `${event.after.ivs.attack}/${event.after.ivs.defense}/${event.after.ivs.hp}`
                  : "Unknown"}
                {event.ivsSource ? ` (${event.ivsSource}, user confirmed)` : ""}
                .
              </p>
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
