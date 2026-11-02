import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { nativeErrorMessage } from "./native";

export interface ReferenceStatus {
  active: {
    provider: string;
    sourceSha256: string;
    cpMultipliers: [number, number][];
    combat?: { version: string; moves: unknown[]; pools: unknown[] } | null;
    preparation?: {
      version: string;
      powerUpCosts: unknown[];
      evolutions: unknown[];
      moveAccess: unknown[];
    } | null;
    trainerCombat?: {
      version: string;
      moves: unknown[];
      pools: unknown[];
    } | null;
    coverage: string[];
    limitations: string[];
    artifacts?: {
      kind: string;
      sourceUrl: string;
      sha256: string;
      upstreamRevision: string;
      retrievedAt: string;
    }[];
    catalog: {
      dataset: {
        datasetId: string;
        version: string;
        sourceUrl: string;
        upstreamRevision: string;
        retrievedAt: string;
        license: string;
        notice: string;
      };
      species: unknown[];
      forms: unknown[];
    };
  } | null;
  canRollback: boolean;
  error?: string | null;
}

export function ReferenceSettings() {
  const [status, setStatus] = useState<ReferenceStatus | null>(null);
  const [provider, setProvider] = useState("pvpoke");
  const [revision, setRevision] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  useEffect(() => {
    let live = true;
    void invoke<ReferenceStatus>("reference_status")
      .then((next) => {
        if (live) setStatus(next);
      })
      .catch((failure: unknown) => {
        if (live)
          setError(
            nativeErrorMessage(
              failure,
              "Unable to load local reference settings.",
            ),
          );
      });
    return () => {
      live = false;
    };
  }, []);
  async function install(rollback = false) {
    if (busy) return;
    setBusy(true);
    setError("");
    setMessage("");
    try {
      const next = rollback
        ? await invoke<ReferenceStatus>("rollback_reference")
        : await invoke<ReferenceStatus | null>("import_reference", {
            provider,
            revision,
          });
      if (next) {
        setStatus(next);
        setMessage(
          "Local reference activated. Inventory identities remain pinned; assessments recalculate when opened.",
        );
      } else setMessage("Import cancelled. Active reference retained.");
    } catch (failure) {
      setError(
        nativeErrorMessage(
          failure,
          "Reference import failed. The active data is retained.",
        ),
      );
    } finally {
      setBusy(false);
    }
  }
  async function exportBackup() {
    if (busy) return;
    setBusy(true);
    setError("");
    setMessage("");
    try {
      const exported = await invoke<boolean>("export_structured_backup");
      setMessage(
        exported
          ? "Structured inventory and plans exported without screenshots or downloaded catalog files. Backup import is not yet supported."
          : "Export cancelled. Existing data retained.",
      );
    } catch (failure) {
      setError(
        nativeErrorMessage(
          failure,
          "Unable to export the structured inventory.",
        ),
      );
    } finally {
      setBusy(false);
    }
  }
  const active = status?.active;
  return (
    <section aria-labelledby="reference-settings-title">
      <h2 id="reference-settings-title">Local reference data</h2>
      <button type="button" disabled={busy} onClick={() => void exportBackup()}>
        Export structured inventory and plans
      </button>
      <p>
        Import a JSON source you have obtained and reviewed. The application
        makes no network requests. Game data is stored in your application-data
        directory and is never included in backups or releases by default.
      </p>
      <p>
        Public access and a software license do not establish rights over the
        underlying Pokémon GO data. Review the provider’s terms and notices for
        your intended use.
      </p>
      <fieldset disabled={busy}>
        <legend>Explicit offline import</legend>
        <label htmlFor="reference-provider">Provider / file format</label>
        <select
          id="reference-provider"
          value={provider}
          onChange={(e) => setProvider(e.target.value)}
        >
          <option value="pvpoke">PvPoke — gamemaster.json</option>
          <option value="pokeminers">
            PokeMiners — latest.json Game Master
          </option>
          <option value="pogoapi">
            PoGoAPI — source files or local JSON bundle
          </option>
          <option value="snapshot">Reviewed Task 007 schema-1 snapshot</option>
        </select>
        <p>
          For PoGoAPI, select the downloaded pokemon_stats, pokemon_types and
          cp_multiplier JSON files together, or one existing bundle. For gym
          attack potential also select fast_moves, charged_moves,
          current_pokemon_moves and type_effectiveness. Add pokemon_evolutions,
          pokemon_powerup_requirements, pvp_fast_moves and pvp_charged_moves for
          evolution, Power Up, move acquisition and Trainer-Battle role signals
          (up to eleven files). Individual observations remain manual and
          optional.
        </p>
        <p>
          Get files from{" "}
          <a href="https://pogoapi.net/api/v1/pokemon_stats.json">
            PoGoAPI stats
          </a>
          , <a href="https://pogoapi.net/api/v1/pokemon_types.json">types</a>{" "}
          and{" "}
          <a href="https://pogoapi.net/api/v1/cp_multiplier.json">
            CP multipliers
          </a>
          , <a href="https://pogoapi.net/api/v1/fast_moves.json">fast moves</a>,{" "}
          <a href="https://pogoapi.net/api/v1/charged_moves.json">
            charged moves
          </a>
          ,{" "}
          <a href="https://pogoapi.net/api/v1/current_pokemon_moves.json">
            current move pools
          </a>
          ,{" "}
          <a href="https://pogoapi.net/api/v1/type_effectiveness.json">
            type effectiveness
          </a>
          ,{" "}
          <a href="https://pogoapi.net/api/v1/pokemon_evolutions.json">
            evolutions
          </a>
          ,{" "}
          <a href="https://pogoapi.net/api/v1/pokemon_powerup_requirements.json">
            Power Up requirements
          </a>
          ,{" "}
          <a href="https://pogoapi.net/api/v1/pvp_fast_moves.json">
            Trainer fast moves
          </a>
          ,{" "}
          <a href="https://pogoapi.net/api/v1/pvp_charged_moves.json">
            Trainer charged moves
          </a>
          , or the{" "}
          <a href="https://github.com/pvpoke/pvpoke/tree/master/src/data/gamemaster">
            PvPoke
          </a>
          /<a href="https://github.com/PokeMiners/game_masters">PokeMiners</a>{" "}
          repositories. You initiate and manage downloads outside this
          application.
        </p>
        <label htmlFor="reference-revision">
          Upstream commit or dated source identifier
        </label>
        <p>
          Use a new revision label when adding files to a previously imported
          reference.
        </p>
        <input
          id="reference-revision"
          value={revision}
          maxLength={100}
          onChange={(e) => setRevision(e.target.value)}
        />
        <button
          type="button"
          disabled={!revision.trim()}
          onClick={() => void install()}
        >
          Select and activate reference JSON
        </button>
        <button
          type="button"
          disabled={!status?.canRollback}
          onClick={() => void install(true)}
        >
          Roll back to previous reference
        </button>
      </fieldset>
      {busy && <p role="status">Validating local reference…</p>}
      {error && <p role="alert">{error}</p>}
      {status?.error && <p role="alert">{status.error}</p>}
      {message && <p role="status">{message}</p>}
      {active ? (
        <>
          <h3>
            Active: {active.catalog.dataset.datasetId} /{" "}
            {active.catalog.dataset.version}
          </h3>
          <p>
            {active.catalog.species.length} species ·{" "}
            {active.catalog.forms.length} forms · {active.cpMultipliers.length}{" "}
            CP multipliers
          </p>
          <p>
            {active.combat
              ? `Gym reference: ${active.combat.moves.length} supported moves · ${active.combat.pools.length} ordinary-form pools · ${active.combat.version}`
              : "Gym combat reference unavailable: import the four supplemental combat files with the catalog to assess attack potential."}
          </p>
          <p>
            {active.preparation
              ? `Preparation reference: ${active.preparation.evolutions.length} evolution branches · ${active.preparation.powerUpCosts.length} Power Up intervals · ${active.preparation.moveAccess.length} move-acquisition lists`
              : "Preparation mechanics unavailable: add evolution and Power Up source files."}
          </p>
          <p>
            {active.trainerCombat
              ? `Trainer-Battle reference: ${active.trainerCombat.moves.length} moves · ${active.trainerCombat.pools.length} pools`
              : "Trainer-Battle mechanics unavailable: add both PvP move source files."}
          </p>
          <p>
            Source:{" "}
            <a href={active.catalog.dataset.sourceUrl}>{active.provider}</a> ·
            revision {active.catalog.dataset.upstreamRevision} · imported{" "}
            {active.catalog.dataset.retrievedAt}
          </p>
          <p>Declared license: {active.catalog.dataset.license}</p>
          <p>{active.catalog.dataset.notice}</p>
          <p className="digest">Source SHA-256: {active.sourceSha256}</p>
          {active.artifacts && (
            <details>
              <summary>Individual source artifacts and provenance</summary>
              <ul>
                {active.artifacts.map((artifact) => (
                  <li key={artifact.kind}>
                    <a href={artifact.sourceUrl}>{artifact.kind}</a> · declared
                    revision {artifact.upstreamRevision} ·{" "}
                    {artifact.retrievedAt}
                    <p className="digest">Raw SHA-256: {artifact.sha256}</p>
                  </li>
                ))}
              </ul>
            </details>
          )}
          <h3>Supported data</h3>
          <ul>
            {active.coverage.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
          <h3>Limitations</h3>
          <ul>
            {active.limitations.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </>
      ) : (
        <p>
          No user reference installed. Confirmed observations and manual Power
          Up remain available offline.
        </p>
      )}
    </section>
  );
}
