import { useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { nativeErrorMessage, type InventoryRecord, type Tag } from "./native";

export function TagEditor({
  record,
  tags,
  disabled,
  onSaved,
  onActiveChange,
}: {
  record: InventoryRecord;
  tags: Tag[];
  disabled: boolean;
  onSaved: () => Promise<void>;
  onActiveChange: (active: boolean) => void;
}) {
  const [draft, setDraft] = useState<string[] | null>(null);
  const [expected, setExpected] = useState<string[]>([]);
  const [name, setName] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const pending = useRef(false);
  async function save(create = false) {
    if (pending.current || !draft) return;
    pending.current = true;
    setBusy(true);
    setError("");
    try {
      if (create) {
        await invoke("create_custom_tag", { name });
        setName("");
        await onSaved();
      } else {
        await invoke("confirm_pokemon_tags", {
          id: record.id,
          expected,
          desired: draft,
        });
        await onSaved();
        setDraft(null);
        onActiveChange(false);
      }
    } catch (failure) {
      setError(
        nativeErrorMessage(
          failure,
          "Unable to save tags. Cancel and reload after conflicting changes.",
        ),
      );
    } finally {
      pending.current = false;
      setBusy(false);
    }
  }
  return (
    <section aria-labelledby="tag-editor-title">
      <h3 id="tag-editor-title">Applied tags</h3>
      <p>
        Suggestions never change these user-applied annotations. SAVE/SPECIAL
        protects retention; custom tag names carry no inferred strategy.
      </p>
      {draft === null ? (
        <button
          type="button"
          disabled={disabled}
          onClick={() => {
            setDraft(record.tags.map((t) => t.id));
            setExpected(record.tags.map((t) => t.id));
            setError("");
            onActiveChange(true);
          }}
        >
          Edit applied tags
        </button>
      ) : (
        <fieldset disabled={busy}>
          <legend>Explicit tag assignments</legend>
          {tags.map((tag) => (
            <label key={tag.id}>
              <input
                type="checkbox"
                checked={draft.includes(tag.id)}
                onChange={(e) =>
                  setDraft(
                    e.target.checked
                      ? [...draft, tag.id]
                      : draft.filter((id) => id !== tag.id),
                  )
                }
              />
              {tag.name}
            </label>
          ))}
          <label>
            New custom tag
            <input
              value={name}
              maxLength={50}
              onChange={(e) => setName(e.target.value)}
            />
          </label>
          <button
            type="button"
            disabled={!name.trim()}
            onClick={() => void save(true)}
          >
            Create custom tag
          </button>
          <button type="button" onClick={() => void save()}>
            Confirm applied tags
          </button>
          <button
            type="button"
            onClick={() => {
              setDraft(null);
              setError("");
              onActiveChange(false);
            }}
          >
            Cancel tag changes
          </button>
        </fieldset>
      )}
      {error && <p role="alert">{error}</p>}
    </section>
  );
}
