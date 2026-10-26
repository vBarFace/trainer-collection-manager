import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  nativeErrorMessage,
  type ReferenceIdentity,
  type ReferenceResolution,
} from "./native";

const resolutionMessages: Record<ReferenceResolution["state"], string> = {
  not_set: "Unresolved: no species or form has been confirmed.",
  unavailable:
    "Reference data unavailable. This release includes an empty catalog while dataset redistribution rights are reviewed. Your confirmed identity is retained.",
  unknown_species:
    "Unresolved: the confirmed species is absent from this catalog.",
  unknown_form:
    "Unresolved: confirm a supported form for this species. No default form is assumed.",
  ambiguous:
    "Unresolved: this identity matches multiple catalog entries. Correct the species or form using a stable identifier.",
  resolved:
    "Reference identity resolved from manually confirmed species and form.",
};

export function ReferencePanel({
  reference,
  busy,
  saving,
  onSave,
  onReload,
}: {
  reference: ReferenceResolution;
  busy: boolean;
  saving: boolean;
  onSave: (
    expected: ReferenceIdentity | null,
    identity: ReferenceIdentity | null,
  ) => Promise<void>;
  onReload: () => void;
}) {
  const [editing, setEditing] = useState(false);
  const [status, setStatus] = useState("");
  const editButton = useRef<HTMLButtonElement>(null);
  const { species, form, dataset } = reference;
  useEffect(() => {
    if (!editing && status) editButton.current?.focus();
  }, [editing, status]);

  return (
    <section aria-labelledby="reference-title" aria-busy={saving}>
      <h3 id="reference-title">Pokémon reference</h3>
      <p>{resolutionMessages[reference.state]}</p>
      <p>
        Confirmed species: {reference.identity?.species ?? "Unknown"} ·
        Confirmed form: {reference.identity?.form ?? "Unknown"}
      </p>
      <p>
        Display names and OCR suggestions do not confirm species or form.
        Reference metadata is separate from your saved CP, IVs and tags.
      </p>
      {species && (
        <p>
          National Pokédex: #{species.dexNumber} · Species: {species.name} (
          {species.id})
        </p>
      )}
      {form && (
        <>
          <p>
            Form: {form.name} ({form.id})
          </p>
          <p>Types: {form.types?.join(" / ") ?? "Unknown"}</p>
          <p>
            GO base stats:{" "}
            {form.baseStats
              ? `${form.baseStats.attack} attack / ${form.baseStats.defense} defense / ${form.baseStats.stamina} stamina`
              : "Unknown"}
          </p>
          {form.evolutions === null ? (
            <p>Evolution information: Unknown</p>
          ) : form.evolutions.length === 0 ? (
            <p>Evolution information: No evolution recorded in this dataset.</p>
          ) : (
            <ul aria-label="Reference evolutions">
              {form.evolutions.map((evolution) => (
                <li key={evolution.targetFormId}>
                  Target form: {evolution.targetFormId}. Requirements:{" "}
                  {evolution.requirements === null
                    ? "Unknown"
                    : evolution.requirements.length
                      ? evolution.requirements.join(", ")
                      : "None recorded"}
                </li>
              ))}
            </ul>
          )}
        </>
      )}
      <p>
        Dataset: {dataset.datasetId} · Version: {dataset.version}
      </p>
      <p>{dataset.notice}</p>
      <details>
        <summary>Reference provenance</summary>
        <p>Source: {dataset.sourceUrl}</p>
        <p>Upstream revision: {dataset.upstreamRevision}</p>
        <p>Retrieved: {dataset.retrievedAt}</p>
        <p>License: {dataset.license}</p>
        <p>Rights evidence: {dataset.rightsEvidenceUrl}</p>
      </details>
      <button
        ref={editButton}
        type="button"
        disabled={busy || saving || editing}
        onClick={() => {
          setStatus("");
          setEditing(true);
        }}
      >
        Correct species and form
      </button>
      {editing && (
        <IdentityForm
          identity={reference.identity}
          busy={busy || saving}
          saving={saving}
          onSave={onSave}
          onSaved={() => {
            setEditing(false);
            setStatus("Species and form confirmation saved locally.");
          }}
          onCancel={() => {
            setEditing(false);
            setStatus("Identity correction cancelled. Nothing was saved.");
          }}
          onReload={() => {
            setEditing(false);
            setStatus("");
            onReload();
          }}
        />
      )}
      {status && <p role="status">{status}</p>}
    </section>
  );
}

function IdentityForm({
  identity,
  busy,
  saving,
  onSave,
  onSaved,
  onCancel,
  onReload,
}: {
  identity: ReferenceIdentity | null;
  busy: boolean;
  saving: boolean;
  onSave: (
    expected: ReferenceIdentity | null,
    identity: ReferenceIdentity | null,
  ) => Promise<void>;
  onSaved: () => void;
  onCancel: () => void;
  onReload: () => void;
}) {
  const expected = useRef(identity);
  const [species, setSpecies] = useState(identity?.species ?? "");
  const [form, setForm] = useState(identity?.form ?? "");
  const [error, setError] = useState("");
  const [requiresReload, setRequiresReload] = useState(false);
  const submitting = useRef(false);
  const mounted = useRef(false);
  const speciesInput = useRef<HTMLInputElement>(null);
  useEffect(() => {
    mounted.current = true;
    speciesInput.current?.focus();
    return () => {
      mounted.current = false;
    };
  }, []);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (submitting.current || busy || requiresReload) return;
    if (
      (!species && form) ||
      [species, form].some(
        (value) =>
          value.trim() !== value ||
          Array.from(value).length > 100 ||
          /\p{Cc}/u.test(value),
      )
    ) {
      setError(
        "Enter a species before a form. Each value must contain at most 100 characters, without surrounding whitespace or control characters.",
      );
      return;
    }
    submitting.current = true;
    setError("");
    try {
      await onSave(
        expected.current,
        species ? { species, form: form || null } : null,
      );
      if (mounted.current) onSaved();
    } catch (failure) {
      if (mounted.current) {
        setError(
          nativeErrorMessage(
            failure,
            "Unable to save species and form. Your draft is retained; try again.",
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
      noValidate
      aria-labelledby="identity-title"
    >
      <h4 id="identity-title">Confirm species and form</h4>
      <p id="identity-help">
        Enter an exact catalog name, alias or stable identifier. Leave form
        empty if unknown, or both fields empty to clear identity. Unknown
        entries are retained without guessing. Confirm only what you have
        verified; nothing changes until you submit. Selecting another record
        discards this draft.
      </p>
      {error && (
        <p role="alert" id="identity-error">
          {error}
        </p>
      )}
      <fieldset disabled={busy || requiresReload}>
        <legend>Manually confirmed identity</legend>
        <label htmlFor="identity-species">Species</label>
        <input
          id="identity-species"
          ref={speciesInput}
          autoComplete="off"
          value={species}
          onChange={(event) => setSpecies(event.target.value)}
          aria-describedby={`identity-help${error ? " identity-error" : ""}`}
        />
        <label htmlFor="identity-form">Form</label>
        <input
          id="identity-form"
          autoComplete="off"
          value={form}
          onChange={(event) => setForm(event.target.value)}
          aria-describedby={`identity-help${error ? " identity-error" : ""}`}
        />
        <button type="submit">
          {saving ? "Saving identity…" : "Confirm identity correction"}
        </button>
      </fieldset>
      <button type="button" disabled={busy} onClick={onCancel}>
        Cancel identity correction
      </button>
      {requiresReload && (
        <button type="button" disabled={busy} onClick={onReload}>
          Discard identity draft and reload inventory
        </button>
      )}
      {saving && <p role="status">Saving confirmed identity locally…</p>}
    </form>
  );
}
