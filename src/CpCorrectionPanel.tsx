import { useEffect, useRef, useState, type FormEvent } from "react";
import { nativeErrorMessage, type InventoryRecord } from "./native";
import { validateCp } from "./review";

type SaveCp = (
  expected: string | null,
  revision: string,
  cp: string | null,
) => Promise<void>;

export function CpCorrectionPanel({
  record,
  busy,
  saving,
  onSave,
  onReload,
}: {
  record: InventoryRecord;
  busy: boolean;
  saving: boolean;
  onSave: SaveCp;
  onReload: () => Promise<void>;
}) {
  const [editing, setEditing] = useState(false);
  const [status, setStatus] = useState("");
  const editButton = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!editing && status) editButton.current?.focus();
  }, [editing, status]);

  return (
    <section aria-labelledby="cp-title" aria-busy={saving}>
      <h3 id="cp-title">Saved CP</h3>
      <p>Confirmed CP: {record.cp ?? "Unknown"}</p>
      <button
        ref={editButton}
        type="button"
        disabled={busy || saving || editing}
        onClick={() => {
          setStatus("");
          setEditing(true);
        }}
      >
        Correct saved CP
      </button>
      {editing && (
        <CpForm
          record={record}
          busy={busy || saving}
          saving={saving}
          onSave={onSave}
          onSaved={() => {
            setEditing(false);
            setStatus("CP correction saved locally.");
          }}
          onCancel={() => {
            setEditing(false);
            setStatus("CP correction cancelled. Nothing was saved.");
          }}
          onReload={async () => {
            await onReload();
            setEditing(false);
            setStatus(
              "Inventory reloaded. Open a new CP correction to continue.",
            );
          }}
        />
      )}
      {status && <p role="status">{status}</p>}
    </section>
  );
}

function CpForm({
  record,
  busy,
  saving,
  onSave,
  onSaved,
  onCancel,
  onReload,
}: {
  record: InventoryRecord;
  busy: boolean;
  saving: boolean;
  onSave: SaveCp;
  onSaved: () => void;
  onCancel: () => void;
  onReload: () => Promise<void>;
}) {
  const expected = useRef({
    cp: record.cp,
    revision: record.strategicObservationRevision,
  });
  const [cp, setCp] = useState(record.cp ?? "");
  const [error, setError] = useState("");
  const [requiresReload, setRequiresReload] = useState(false);
  const [reloading, setReloading] = useState(false);
  const submitting = useRef(false);
  const mounted = useRef(false);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    mounted.current = true;
    input.current?.focus();
    return () => {
      mounted.current = false;
    };
  }, []);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (submitting.current || busy || requiresReload) return;
    const invalid = validateCp(cp);
    if (invalid) {
      setError(invalid);
      return;
    }
    submitting.current = true;
    setError("");
    try {
      await onSave(expected.current.cp, expected.current.revision, cp || null);
      if (mounted.current) onSaved();
    } catch (failure) {
      if (mounted.current) {
        const reload =
          typeof failure === "object" &&
          failure !== null &&
          "code" in failure &&
          (failure.code === "conflict" || failure.code === "missing_pokemon");
        setRequiresReload(reload);
        setError(
          reload
            ? "The Pokémon has changed since this form was opened. Reload its information before trying again."
            : nativeErrorMessage(
                failure,
                "Unable to save CP corrections. Your draft is retained; try again.",
              ),
        );
      }
    } finally {
      submitting.current = false;
    }
  }

  async function reload() {
    if (submitting.current || reloading) return;
    submitting.current = true;
    setReloading(true);
    try {
      await onReload();
    } catch (failure) {
      if (mounted.current)
        setError(
          nativeErrorMessage(failure, "Unable to reload. Try reloading again."),
        );
    } finally {
      submitting.current = false;
      if (mounted.current) setReloading(false);
    }
  }

  return (
    <form
      onSubmit={(event) => void submit(event)}
      aria-labelledby="cp-correction-title"
      aria-busy={saving || reloading}
      noValidate
    >
      <h4 id="cp-correction-title">Correct saved CP</h4>
      <p>Saved CP: {expected.current.cp ?? "Unknown"}.</p>
      <p id="cp-correction-help">
        Enter a positive whole number, or leave empty to mark CP unknown.
        Nothing changes until you confirm. This corrects observed data and does
        not record a Power Up, infer level or change appraisal results or tags.
        Selecting another record discards unsaved corrections.
      </p>
      {error && (
        <p role="alert" id="cp-correction-error">
          {error}
        </p>
      )}
      <fieldset disabled={busy || requiresReload || reloading}>
        <legend>Corrected CP observation</legend>
        <label htmlFor="correct-cp">Corrected CP</label>
        <input
          id="correct-cp"
          ref={input}
          type="text"
          inputMode="numeric"
          autoComplete="off"
          value={cp}
          onChange={(event) => setCp(event.target.value)}
          aria-describedby={`cp-correction-help${error ? " cp-correction-error" : ""}`}
        />
        <button type="button" onClick={() => setCp("")}>
          Mark CP unknown
        </button>{" "}
        <button type="submit">
          {saving ? "Saving CP correction…" : "Confirm CP correction"}
        </button>
      </fieldset>
      <button type="button" onClick={onCancel} disabled={busy || reloading}>
        Cancel CP correction
      </button>
      {requiresReload && (
        <button
          type="button"
          disabled={reloading || busy}
          onClick={() => void reload()}
        >
          Discard CP corrections and reload inventory
        </button>
      )}
      {saving && <p role="status">Saving confirmed CP correction locally…</p>}
      {reloading && <p role="status">Reloading inventory…</p>}
    </form>
  );
}
