import { useEffect, useRef, useState, type FormEvent } from "react";
import { native, nativeErrorMessage, type InventoryRecord } from "./native";
import { validateCp } from "./review";

export function PowerUpPanel({
  record,
  busy,
  onCommit,
  onActiveChange,
  onReload,
}: {
  record: InventoryRecord;
  busy: boolean;
  onCommit: (action: () => Promise<InventoryRecord>) => Promise<void>;
  onActiveChange: (active: boolean) => void;
  onReload: () => Promise<void>;
}) {
  const [draft, setDraft] = useState<InventoryRecord | null>(null);
  const [cp, setCp] = useState("");
  const [reviewing, setReviewing] = useState(false);
  const [confirmed, setConfirmed] = useState(false);
  const [working, setWorking] = useState(false);
  const [stale, setStale] = useState(false);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const request = useRef("");
  const pending = useRef(false);
  const alive = useRef(false);
  const button = useRef<HTMLButtonElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);
  useEffect(() => {
    if (draft) {
      if (reviewing) heading.current?.focus();
      else input.current?.focus();
    }
  }, [draft, reviewing]);
  useEffect(() => {
    if (!draft && !working && status) button.current?.focus();
  }, [draft, working, status]);
  function close(message: string) {
    setDraft(null);
    setCp("");
    setReviewing(false);
    setConfirmed(false);
    setError("");
    setStale(false);
    setStatus(message);
    onActiveChange(false);
  }
  function begin() {
    if (busy || pending.current || draft) return;
    request.current = crypto.randomUUID();
    setDraft(structuredClone(record));
    setCp("");
    setReviewing(false);
    setConfirmed(false);
    setError("");
    setStale(false);
    setStatus("");
    onActiveChange(true);
  }
  async function submit(event: FormEvent) {
    event.preventDefault();
    if (busy || pending.current || stale || !draft) return;
    const invalid = validateCp(cp);
    if (invalid) {
      setError(invalid);
      return;
    }
    if (!reviewing) {
      setError("");
      setReviewing(true);
      return;
    }
    if (!confirmed) return;
    pending.current = true;
    setWorking(true);
    setError("");
    try {
      await onCommit(() =>
        native.confirmPowerUp(draft.id, {
          requestId: request.current,
          expectedStrategicObservationRevision:
            draft.strategicObservationRevision,
          cp: cp || null,
          performedInGameConfirmed: confirmed,
        }),
      );
      if (alive.current) close("Power Up confirmed and saved locally.");
    } catch (failure) {
      if (alive.current) {
        setError(
          nativeErrorMessage(
            failure,
            "Unable to save Power Up. Your review is retained; try again.",
          ),
        );
        setStale(
          typeof failure === "object" &&
            failure !== null &&
            "code" in failure &&
            (failure.code === "conflict" ||
              failure.code === "missing_pokemon" ||
              failure.code === "already_confirmed"),
        );
      }
    } finally {
      pending.current = false;
      if (alive.current) setWorking(false);
    }
  }
  async function reload() {
    if (pending.current || busy) return;
    pending.current = true;
    setWorking(true);
    try {
      await onReload();
      if (alive.current)
        close("Inventory reloaded. Start a new Power Up review.");
      else onActiveChange(false);
    } catch (failure) {
      if (alive.current)
        setError(nativeErrorMessage(failure, "Unable to reload inventory."));
    } finally {
      pending.current = false;
      if (alive.current) setWorking(false);
    }
  }
  const difference =
    reviewing && draft?.cp !== null && draft?.cp !== undefined && cp
      ? BigInt(cp) - BigInt(draft.cp)
      : null;
  return (
    <section aria-labelledby="power-up-title" aria-busy={working}>
      <h3 id="power-up-title">Confirmed Power Up</h3>
      <p>Record a Power Up you performed in Pokémon GO.</p>
      <button
        ref={button}
        type="button"
        disabled={busy || working || draft !== null}
        onClick={begin}
      >
        Power Up
      </button>
      {draft && (
        <form onSubmit={(event) => void submit(event)} noValidate>
          <p>
            Selected individual: {draft.displayName} · UUID: {draft.id}
          </p>
          <p>
            Species: {draft.reference.identity?.species ?? "Unknown"} · Form:{" "}
            {draft.reference.identity?.form ?? "Unspecified"}
          </p>
          <p>Pending action · Nothing is saved until confirmation.</p>
          <fieldset disabled={busy || working || stale}>
            <legend>
              {reviewing ? "Review Power Up" : "Power Up observation"}
            </legend>
            {!reviewing ? (
              <>
                <label htmlFor="power-up-cp">Updated CP (optional)</label>
                <input
                  ref={input}
                  id="power-up-cp"
                  type="text"
                  inputMode="numeric"
                  autoComplete="off"
                  value={cp}
                  onChange={(event) => setCp(event.target.value)}
                  aria-describedby="power-up-help"
                />
                <p id="power-up-help">
                  Enter the updated CP as a positive whole number, or leave
                  blank if unknown. Blank keeps the saved CP, including an
                  unknown value. No screenshot is required.
                </p>
                <button type="submit">Review Power Up</button>
              </>
            ) : (
              <>
                <h4 ref={heading} tabIndex={-1}>
                  Review proposed changes
                </h4>
                <p>Saved CP: {draft.cp ?? "Unknown"}</p>
                <p>
                  CP after confirmation: {cp || draft.cp || "Unknown"}
                  {cp
                    ? " (manually entered)"
                    : " (saved CP kept; no new CP supplied)"}
                </p>
                {difference !== null && (
                  <p>
                    CP difference: {difference >= 0n ? "+" : ""}
                    {difference.toString()}
                  </p>
                )}
                <p>
                  The same UUID, species, form, IVs, tags and unrelated
                  attributes are preserved. No level or resource costs are
                  inferred.
                </p>
                <label>
                  <input
                    type="checkbox"
                    checked={confirmed}
                    onChange={(event) => setConfirmed(event.target.checked)}
                  />
                  I confirm I performed this Power Up in Pokémon GO
                </label>
                <button type="submit" disabled={!confirmed}>
                  Confirm Power Up
                </button>{" "}
                <button
                  type="button"
                  onClick={() => {
                    setReviewing(false);
                    setConfirmed(false);
                    setError("");
                  }}
                >
                  Edit Power Up
                </button>
              </>
            )}
          </fieldset>
          <button
            type="button"
            disabled={busy || working}
            onClick={() => close("Power Up cancelled. Nothing was saved.")}
          >
            Cancel Power Up
          </button>
        </form>
      )}
      {error && <p role="alert">{error}</p>}
      {stale && (
        <button
          type="button"
          disabled={busy || working}
          onClick={() => void reload()}
        >
          Reload current individual
        </button>
      )}
      {working && <p role="status">Saving or reloading confirmed Power Up…</p>}
      {status && <p role="status">{status}</p>}
    </section>
  );
}
