import { useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { nativeErrorMessage, type InventoryRecord } from "./native";

export interface IndividualFacts {
  currentHp: number | null;
  maximumHp: number | null;
  weightKg: number | null;
  heightM: number | null;
  gender: string | null;
  catchDate: string | null;
  fastMove: string | null;
  chargedMove: string | null;
  secondChargedMove: string | null;
  level: number | null;
  levelEvidence: string | null;
  shiny: boolean | null;
  shadow: boolean | null;
  purified: boolean | null;
  favorite: boolean | null;
  protected: boolean;
}
interface DetailsView {
  values: IndividualFacts;
  observationRevision: string;
  needsReconfirmation: boolean;
}
const numberFields = [
  "currentHp",
  "maximumHp",
  "weightKg",
  "heightM",
  "level",
] as const;
const textFields = [
  "gender",
  "catchDate",
  "fastMove",
  "chargedMove",
  "secondChargedMove",
  "levelEvidence",
] as const;
const labels: Record<
  (typeof numberFields)[number] | (typeof textFields)[number],
  string
> = {
  currentHp: "Current HP",
  maximumHp: "Maximum HP",
  weightKg: "Weight (kg)",
  heightM: "Height (m)",
  level: "Level (only with evidence)",
  gender: "Explicit gender",
  catchDate: "Catch date (YYYY-MM-DD)",
  fastMove: "Fast move",
  chargedMove: "Charged move",
  secondChargedMove: "Second charged move",
  levelEvidence: "Level evidence",
};
export function IndividualDetails({
  record,
  disabled,
  onSaved,
  onActiveChange,
}: {
  record: InventoryRecord;
  disabled: boolean;
  onSaved: () => Promise<void>;
  onActiveChange: (active: boolean) => void;
}) {
  const [view, setView] = useState<DetailsView | null>(null);
  const [draft, setDraft] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const pending = useRef(false);
  async function open() {
    if (pending.current || disabled) return;
    pending.current = true;
    setBusy(true);
    setError("");
    setMessage("");
    onActiveChange(true);
    try {
      const next = await invoke<DetailsView>("individual_details", {
        id: record.id,
      });
      setView(next);
      setDraft(
        Object.fromEntries(
          Object.entries(next.values).map(([key, value]) => [
            key,
            value === null ? "" : String(value),
          ]),
        ),
      );
    } catch (failure) {
      setError(
        nativeErrorMessage(failure, "Unable to load individual details."),
      );
      onActiveChange(false);
    } finally {
      setBusy(false);
      pending.current = false;
    }
  }
  async function save() {
    if (!view || pending.current) return;
    pending.current = true;
    setBusy(true);
    setError("");
    try {
      if (
        numberFields.some(
          (key) => draft[key] !== "" && !Number.isFinite(Number(draft[key])),
        )
      ) {
        setError(
          "Numeric observations must be valid numbers, or empty for unknown.",
        );
        return;
      }
      const values = Object.fromEntries(
        Object.entries(draft).map(([key, value]) => [
          key,
          value === ""
            ? null
            : numberFields.includes(key as (typeof numberFields)[number])
              ? Number(value)
              : [
                    "shiny",
                    "shadow",
                    "purified",
                    "favorite",
                    "protected",
                  ].includes(key)
                ? value === "true"
                : value,
        ]),
      );
      await invoke("save_individual_details", {
        id: record.id,
        expectedRevision: view.observationRevision,
        values,
      });
      await onSaved();
      setView(null);
      onActiveChange(false);
      setMessage(
        "Optional observations confirmed locally. Blank fields remain unknown.",
      );
    } catch (failure) {
      setError(
        nativeErrorMessage(
          failure,
          "Unable to save details. Reload the individual after a conflict.",
        ),
      );
    } finally {
      setBusy(false);
      pending.current = false;
    }
  }
  return (
    <section aria-labelledby="individual-details-title">
      <h3 id="individual-details-title">
        Individual observations and protection
      </h3>
      {!view && (
        <button
          type="button"
          disabled={disabled || busy}
          onClick={() => void open()}
        >
          Review optional observations
        </button>
      )}
      {view && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void save();
          }}
        >
          <p>
            Every value below is manually confirmed. Blank means unknown.
            Favoriting and protection are separate choices. Moves are
            observations; legality and event availability require reference
            evidence.
          </p>
          {view.needsReconfirmation && (
            <p role="status">
              The individual changed since these fields were confirmed. Recheck
              values, especially level and moves, before confirming again.
            </p>
          )}
          <fieldset disabled={busy}>
            <legend>Optional confirmed facts</legend>
            {[...numberFields, ...textFields].map((key) => (
              <label key={key}>
                {labels[key]}
                <input
                  value={draft[key] ?? ""}
                  inputMode={
                    numberFields.includes(key as (typeof numberFields)[number])
                      ? "decimal"
                      : undefined
                  }
                  maxLength={100}
                  onChange={(e) =>
                    setDraft({ ...draft, [key]: e.target.value })
                  }
                />
              </label>
            ))}
            {(["shiny", "shadow", "purified", "favorite"] as const).map(
              (key) => (
                <label key={key}>
                  {key[0].toUpperCase() + key.slice(1)}
                  <select
                    value={draft[key] ?? ""}
                    onChange={(e) =>
                      setDraft({ ...draft, [key]: e.target.value })
                    }
                  >
                    <option value="">Unknown</option>
                    <option value="true">Confirmed yes</option>
                    <option value="false">Confirmed no</option>
                  </select>
                </label>
              ),
            )}
            <label>
              <input
                type="checkbox"
                checked={draft.protected === "true"}
                onChange={(e) =>
                  setDraft({ ...draft, protected: String(e.target.checked) })
                }
              />
              Protect this individual from irreversible recommendations
            </label>
            <button type="submit">Confirm optional observations</button>{" "}
            <button
              type="button"
              onClick={() => {
                setView(null);
                onActiveChange(false);
                setError("");
              }}
            >
              Cancel optional observations
            </button>
          </fieldset>
        </form>
      )}
      {error && <p role="alert">{error}</p>}
      {message && <p role="status">{message}</p>}
    </section>
  );
}
