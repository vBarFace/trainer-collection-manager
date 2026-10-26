import { useEffect, useRef, useState, type FormEvent } from "react";
import { nativeErrorMessage, type AppraisalIvs } from "./native";
import { validateIvs } from "./review";

export function IvCorrectionForm({
  ivs,
  busy,
  onSave,
  onSaved,
  onCancel,
  onReload,
}: {
  ivs: AppraisalIvs | null;
  busy: boolean;
  onSave: (
    expected: AppraisalIvs | null,
    corrected: AppraisalIvs | null,
  ) => Promise<void>;
  onSaved: () => void;
  onCancel: () => void;
  onReload: () => void;
}) {
  const expected = useRef(ivs);
  const [fields, setFields] = useState({
    attack: ivs?.attack.toString() ?? "",
    defense: ivs?.defense.toString() ?? "",
    hp: ivs?.hp.toString() ?? "",
  });
  const [error, setError] = useState("");
  const [requiresReload, setRequiresReload] = useState(false);
  const mounted = useRef(false);
  const submitting = useRef(false);
  const attackInput = useRef<HTMLInputElement>(null);
  useEffect(() => {
    mounted.current = true;
    attackInput.current?.focus();
    return () => {
      mounted.current = false;
    };
  }, []);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (submitting.current || busy || requiresReload) return;
    const corrected = validateIvs(fields);
    if (typeof corrected === "string") {
      setError(corrected);
      return;
    }
    submitting.current = true;
    setError("");
    try {
      await onSave(expected.current, corrected);
      if (mounted.current) onSaved();
    } catch (failure) {
      if (mounted.current) {
        setError(
          nativeErrorMessage(
            failure,
            "Unable to save IV corrections. Your draft is retained; try again.",
          ),
        );
        setRequiresReload(
          typeof failure === "object" &&
            failure !== null &&
            "code" in failure &&
            (failure.code === "conflict" || failure.code === "missing_pokemon"),
        );
      }
    } finally {
      submitting.current = false;
    }
  }

  return (
    <form
      onSubmit={(event) => void submit(event)}
      aria-labelledby="correction-title"
      aria-busy={busy}
      noValidate
    >
      <h4 id="correction-title">Correct saved appraisal IVs</h4>
      <p>
        Saved IVs:{" "}
        {expected.current
          ? `${expected.current.attack} / ${expected.current.defense} / ${expected.current.hp} (attack / defense / HP)`
          : "Unknown"}
        .
      </p>
      <p id="correction-help">
        Enter all three whole numbers from 0 to 15, or leave all three empty to
        mark them unknown. HP is the appraisal IV. Nothing changes until you
        confirm. Changing IVs makes a previous appraisal stale; it does not
        evaluate or change tags. Selecting another record discards unsaved
        corrections.
      </p>
      {error && (
        <p role="alert" id="correction-error">
          {error}
        </p>
      )}
      <fieldset disabled={busy || requiresReload}>
        <legend>Corrected IV group</legend>
        {(["attack", "defense", "hp"] as const).map((field) => (
          <div key={field}>
            <label htmlFor={`correct-${field}`}>
              {field === "hp" ? "Corrected HP IV" : `Corrected ${field} IV`}
            </label>
            <input
              id={`correct-${field}`}
              ref={field === "attack" ? attackInput : undefined}
              type="text"
              inputMode="numeric"
              autoComplete="off"
              value={fields[field]}
              onChange={(event) =>
                setFields((current) => ({
                  ...current,
                  [field]: event.target.value,
                }))
              }
              aria-describedby={`correction-help${error ? " correction-error" : ""}`}
            />
          </div>
        ))}
        <button
          type="button"
          onClick={() => setFields({ attack: "", defense: "", hp: "" })}
        >
          Mark all IVs unknown
        </button>{" "}
        <button type="submit">
          {busy ? "Saving IV correction…" : "Confirm IV correction"}
        </button>
      </fieldset>
      <button type="button" onClick={onCancel} disabled={busy}>
        Cancel IV correction
      </button>
      {requiresReload && (
        <button type="button" onClick={onReload}>
          Discard corrections and reload inventory
        </button>
      )}
      {busy && <p role="status">Saving confirmed IV corrections locally…</p>}
    </form>
  );
}
