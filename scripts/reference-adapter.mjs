// Development-only, offline PvPoke adapter. No source data is bundled here.
import { createHash } from "node:crypto";
import {
  readFileSync,
  mkdirSync,
  writeFileSync,
  renameSync,
  rmSync,
} from "node:fs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

export const ADAPTER_VERSION = "pvpoke-1";
const ROOT = fileURLToPath(new URL("../", import.meta.url));
const TYPES = new Set(
  "normal fire water electric grass ice fighting poison ground flying psychic bug rock ghost dragon dark steel fairy".split(
    " ",
  ),
);
// Audited at 38f70320d190e38c2e6c778f572662b4c4e8185d. Unused fields are
// deliberately ignored; new fields on selected records require adapter review.
const SOURCE_FIELDS = new Set(
  "dex speciesName speciesId baseStats types fastMoves chargedMoves tags defaultIVs level25CP buddyDistance thirdMoveCost released family eliteMoves searchPriority nicknames extraChargedMoves legacyMoves levelFloor originalFormId formChange nativeStatBuffs".split(
    " ",
  ),
);
const fail = (message) => {
  throw new Error(message);
};
const object = (v) => v !== null && typeof v === "object" && !Array.isArray(v);
const cmp = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
const sorted = (a, key) => [...a].sort((x, y) => cmp(x[key], y[key]));
export const digest = (bytes) =>
  createHash("sha256").update(bytes).digest("hex");
const canonical = (v) =>
  Array.isArray(v)
    ? v.map(canonical)
    : object(v)
      ? Object.fromEntries(
          Object.keys(v)
            .sort(cmp)
            .map((k) => [k, canonical(v[k])]),
        )
      : v;
export const json = (value) => JSON.stringify(canonical(value), null, 2) + "\n";
function keys(v, required, label) {
  if (
    !object(v) ||
    Object.keys(v).some((k) => !required.includes(k)) ||
    required.some((k) => !Object.hasOwn(v, k))
  )
    fail(`Unsupported ${label} fields`);
}
function text(v, label) {
  if (
    typeof v !== "string" ||
    !v.trim() ||
    v.trim() !== v ||
    /[\p{Cc}]/u.test(v)
  )
    fail(`Invalid ${label}`);
  return v;
}
function id(v) {
  if (typeof v !== "string" || !/^[a-z0-9_:-]{1,100}$/.test(v))
    fail("Invalid stable ID");
  return v;
}
function https(v) {
  text(v, "evidence URL");
  const url = new URL(v);
  if (url.protocol !== "https:" || url.username || url.password)
    fail("Evidence must use HTTPS without credentials");
  return v;
}
function names(v) {
  if (!Array.isArray(v)) fail("Aliases must be an explicit array");
  return v.map((n) => text(n, "alias")).sort(cmp);
}
function integer(v, label) {
  if (!Number.isInteger(v) || v < 1 || v > 65535) fail(`Invalid ${label}`);
  return v;
}
function unique(values, label) {
  if (new Set(values).size !== values.length) fail(`Duplicate ${label}`);
}
function normalizeTypes(v) {
  if (v === null) return null;
  if (!Array.isArray(v) || v.length < 1 || v.length > 2)
    fail("Unsupported upstream types");
  // PvPoke uses `none` only as a second slot, never as an unknown real type.
  const actual = v.length === 2 && v[1] === "none" ? v.slice(0, 1) : [...v];
  if (actual.some((t) => !TYPES.has(t))) fail("Unknown upstream type");
  unique(actual, "type");
  return actual; // Keep primary/secondary semantics.
}
function normalizeStats(v) {
  if (v === null) return null;
  keys(v, ["atk", "def", "hp"], "GO base stats");
  return {
    attack: integer(v.atk, "GO Attack"),
    defense: integer(v.def, "GO Defense"),
    stamina: integer(v.hp, "GO Stamina"),
  };
}

/** Explicit reviewed mappings select a bounded subset; no suffix/name guessing. */
export function transform(source, plan) {
  keys(
    plan,
    ["adapterVersion", "dataset", "rights", "species", "forms"],
    "plan",
  );
  if (plan.adapterVersion !== ADAPTER_VERSION)
    fail("Unsupported adapter version");
  keys(
    plan.dataset,
    [
      "datasetId",
      "version",
      "sourceUrl",
      "upstreamRevision",
      "license",
      "notice",
      "retrievedAt",
      "rightsEvidenceUrl",
    ],
    "dataset",
  );
  const d = plan.dataset;
  id(d.datasetId);
  for (const value of Object.values(d)) text(value, "dataset provenance");
  https(d.sourceUrl);
  https(d.rightsEvidenceUrl);
  if (
    !/^\d{4}-\d{2}-\d{2}$/.test(d.retrievedAt) ||
    new Date(d.retrievedAt).toISOString().slice(0, 10) !== d.retrievedAt
  )
    fail("retrievedAt must be a real YYYY-MM-DD date");
  keys(plan.rights, ["status", "review", "evidenceUrl"], "rights review");
  text(plan.rights.review, "rights review");
  if (https(plan.rights.evidenceUrl) !== d.rightsEvidenceUrl)
    fail("Rights evidence mismatch");
  if (plan.rights.status === "fictional") {
    if (!d.datasetId.startsWith("synthetic-"))
      fail("Fictional data needs synthetic- namespace");
    if (!/^original-[a-z0-9-]+$/.test(d.upstreamRevision))
      fail("Fictional revision must be original-...");
  } else if (plan.rights.status === "reviewed") {
    if (!/^[a-f0-9]{40}$/.test(d.upstreamRevision))
      fail("A full upstream commit SHA is required");
    if (
      d.sourceUrl !==
      `https://raw.githubusercontent.com/pvpoke/pvpoke/${d.upstreamRevision}/src/data/gamemaster/pokemon.json`
    )
      fail("Source URL must identify the pinned PvPoke file");
  } else fail("Dataset rights remain blocked; no candidate generated");
  if (
    !Array.isArray(source) ||
    !Array.isArray(plan.species) ||
    !Array.isArray(plan.forms) ||
    !plan.forms.length
  )
    fail("Expected upstream array and nonempty bounded form mappings");
  const upstream = new Map();
  for (const row of source) {
    if (!object(row)) fail("Expected upstream record");
    id(row.speciesId);
    if (upstream.has(row.speciesId)) fail("Duplicate upstream ID");
    upstream.set(row.speciesId, row);
  }
  const species = sorted(
    plan.species.map((s) => {
      keys(
        s,
        ["id", "dexNumber", "name", "aliases", "evidenceUrl"],
        "species mapping",
      );
      https(s.evidenceUrl);
      return {
        id: id(s.id),
        dexNumber: integer(s.dexNumber, "Dex number"),
        name: text(s.name, "canonical species name"),
        aliases: names(s.aliases),
      };
    }),
    "id",
  );
  unique(
    species.map((s) => s.id),
    "species ID",
  );
  unique(
    species.map((s) => s.dexNumber),
    "Dex number",
  );
  unique(
    plan.forms.map((f) => f.upstreamId),
    "selected upstream ID",
  );
  const forms = sorted(
    plan.forms.map((f) => {
      keys(
        f,
        [
          "upstreamId",
          "id",
          "speciesId",
          "name",
          "aliases",
          "kind",
          "evidenceUrl",
          "evolutions",
        ],
        "form mapping",
      );
      const row = upstream.get(f.upstreamId);
      if (!row) fail(`Selected upstream ID unavailable: ${f.upstreamId}`);
      if (
        Object.keys(row).some((k) => !SOURCE_FIELDS.has(k)) ||
        ["dex", "speciesName", "baseStats", "types", "tags"].some(
          (k) => !Object.hasOwn(row, k),
        )
      )
        fail(`Upstream schema changed: ${f.upstreamId}`);
      text(row.speciesName, "upstream species name");
      if (
        !Array.isArray(row.tags) ||
        row.tags.some((t) => typeof t !== "string")
      )
        fail("Invalid upstream tags");
      if (
        row.tags.some((t) =>
          ["shadow", "purified", "shiny", "lucky"].includes(t),
        ) ||
        /(^|_)(shadow|purified|shiny|lucky)($|_)/.test(row.speciesId)
      )
        fail("Individual attributes cannot become forms");
      if (!["standard", "regional", "other", "mega"].includes(f.kind))
        fail("Unreviewed form kind");
      if (
        row.tags.some((t) => ["mega", "supermega"].includes(t)) !==
        (f.kind === "mega")
      )
        fail("Mega semantics need explicit review");
      const parent = species.find((s) => s.id === f.speciesId);
      if (!parent || parent.dexNumber !== row.dex)
        fail("Form/species Dex mismatch");
      https(f.evidenceUrl);
      let evolutions = null;
      if (f.evolutions !== null) {
        keys(
          f.evolutions,
          ["evidenceUrl", "edges"],
          "reviewed evolution group",
        );
        https(f.evolutions.evidenceUrl);
        if (f.kind === "mega" || !Array.isArray(f.evolutions.edges))
          fail("Unsupported evolution group");
        evolutions = sorted(
          f.evolutions.edges.map((e) => {
            keys(e, ["targetFormId", "requirements"], "evolution edge");
            if (e.requirements !== null && !Array.isArray(e.requirements))
              fail("Invalid requirements");
            return {
              targetFormId: id(e.targetFormId),
              requirements:
                e.requirements === null
                  ? null
                  : e.requirements.map((r) => text(r, "requirement")).sort(cmp),
            };
          }),
          "targetFormId",
        );
      }
      return {
        id: id(f.id),
        speciesId: id(f.speciesId),
        name: text(f.name, "canonical form name"),
        aliases: names(f.aliases),
        types: normalizeTypes(row.types),
        baseStats: normalizeStats(row.baseStats),
        evolutions,
      };
    }),
    "id",
  );
  unique(
    forms.map((f) => f.id),
    "form ID",
  );
  const megaIds = new Set(
    plan.forms.filter((f) => f.kind === "mega").map((f) => f.id),
  );
  if (forms.some((f) => f.evolutions?.some((e) => megaIds.has(e.targetFormId))))
    fail("Mega transformations cannot be evolution targets");
  if (species.some((s) => !forms.some((f) => f.speciesId === s.id)))
    fail("Species without a selected form");
  // Production Catalog::load, not this transformer, owns graph/schema validation.
  return { schemaVersion: 1, dataset: { ...d }, species, forms };
}

/** Carry tombstones forward so removed IDs cannot be recycled in later releases. */
export function provenance(catalog, sourceBytes, planBytes, previous = null) {
  const d = catalog.dataset;
  const outputSha256 = digest(json(catalog));
  let ledger = { species: [], forms: [], versions: [] };
  if (previous !== null) {
    keys(
      previous,
      [
        "adapterVersion",
        "datasetId",
        "upstreamRevision",
        "sourceUrl",
        "retrievedAt",
        "sourceSha256",
        "planSha256",
        "outputSha256",
        "rights",
        "identityLedger",
      ],
      "previous provenance",
    );
    if (
      previous.adapterVersion !== ADAPTER_VERSION ||
      previous.datasetId !== d.datasetId
    )
      fail("Previous provenance namespace/version mismatch");
    for (const field of ["sourceSha256", "planSha256", "outputSha256"])
      if (!/^[a-f0-9]{64}$/.test(previous[field]))
        fail("Invalid previous provenance digest");
    keys(
      previous.identityLedger,
      ["species", "forms", "versions"],
      "identity ledger",
    );
    ledger = structuredClone(previous.identityLedger);
    if (!Object.values(ledger).every(Array.isArray))
      fail("Invalid identity ledger");
    for (const s of ledger.species) {
      keys(s, ["id", "dexNumber"], "species ledger");
      id(s.id);
      integer(s.dexNumber, "ledger Dex");
    }
    for (const f of ledger.forms) {
      keys(f, ["id", "speciesId", "upstreamId", "kind"], "form ledger");
      id(f.id);
      id(f.speciesId);
      id(f.upstreamId);
      text(f.kind, "ledger form kind");
    }
    for (const v of ledger.versions) {
      keys(v, ["version", "outputSha256"], "version ledger");
      text(v.version, "ledger version");
      if (!/^[a-f0-9]{64}$/.test(v.outputSha256))
        fail("Invalid version digest");
    }
    unique(
      ledger.species.map((s) => s.id),
      "ledger species",
    );
    unique(
      ledger.species.map((s) => s.dexNumber),
      "ledger Dex",
    );
    unique(
      ledger.forms.map((f) => f.id),
      "ledger form",
    );
    unique(
      ledger.forms.map((f) => f.upstreamId),
      "ledger upstream ID",
    );
    unique(
      ledger.versions.map((v) => v.version),
      "ledger version",
    );
    if (!ledger.versions.some((v) => v.outputSha256 === previous.outputSha256))
      fail("Previous output digest missing from version ledger");
    if (
      ledger.forms.some(
        (f) => !ledger.species.some((s) => s.id === f.speciesId),
      )
    )
      fail("Dangling previous form identity");
  }
  for (const s of catalog.species) {
    const old = ledger.species.find(
      (v) => v.id === s.id || v.dexNumber === s.dexNumber,
    );
    if (old && (old.id !== s.id || old.dexNumber !== s.dexNumber))
      fail("Species identity conflict; explicit migration review required");
    if (!old) ledger.species.push({ id: s.id, dexNumber: s.dexNumber });
  }
  const plan = JSON.parse(planBytes);
  for (const f of catalog.forms) {
    const mapping = plan.forms.find((m) => m.id === f.id);
    const identity = {
      id: f.id,
      speciesId: f.speciesId,
      upstreamId: mapping.upstreamId,
      kind: mapping.kind,
    };
    const old = ledger.forms.find(
      (v) => v.id === f.id || v.upstreamId === identity.upstreamId,
    );
    if (old && Object.keys(identity).some((k) => identity[k] !== old[k]))
      fail("Form identity conflict; explicit migration review required");
    if (!old) ledger.forms.push(identity);
  }
  const oldVersion = ledger.versions.find((v) => v.version === d.version);
  if (oldVersion && oldVersion.outputSha256 !== outputSha256)
    fail("Snapshot version/content mismatch");
  if (!oldVersion) ledger.versions.push({ version: d.version, outputSha256 });
  ledger = {
    species: sorted(ledger.species, "id"),
    forms: sorted(ledger.forms, "id"),
    versions: sorted(ledger.versions, "version"),
  };
  return {
    adapterVersion: ADAPTER_VERSION,
    datasetId: d.datasetId,
    upstreamRevision: d.upstreamRevision,
    sourceUrl: d.sourceUrl,
    retrievedAt: d.retrievedAt,
    sourceSha256: digest(sourceBytes),
    planSha256: digest(planBytes),
    outputSha256,
    rights: plan.rights,
    identityLedger: ledger,
  };
}

export function validateNative(candidate) {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--features",
      "reference-tools",
      "--bin",
      "validate-reference",
    ],
    {
      cwd: ROOT,
      input: json(candidate),
      encoding: "utf8",
      maxBuffer: 1024 * 1024,
    },
  );
  if (result.error || result.status !== 0)
    fail(`Native validation failed: ${result.error?.message ?? result.stderr}`);
}

export function main(args) {
  const options = {};
  for (let i = 0; i < args.length; i += 2) {
    if (
      !["--source", "--plan", "--out", "--previous"].includes(args[i]) ||
      !args[i + 1] ||
      options[args[i]]
    )
      fail(
        "Usage: --source FILE --plan FILE --out NEW_DIRECTORY [--previous PROVENANCE]",
      );
    options[args[i]] = args[i + 1];
  }
  if (!["--source", "--plan", "--out"].every((k) => options[k]))
    fail("Source, plan and output directory are required");
  const read = (path) => {
    const bytes = readFileSync(path);
    if (bytes.length > 16 * 1024 * 1024) fail("Input exceeds 16 MiB");
    return bytes;
  };
  const sourceBytes = read(options["--source"]);
  const planBytes = read(options["--plan"]);
  const candidate = transform(JSON.parse(sourceBytes), JSON.parse(planBytes));
  const record = provenance(
    candidate,
    sourceBytes,
    planBytes,
    options["--previous"] ? JSON.parse(read(options["--previous"])) : null,
  );
  validateNative(candidate);
  const output = resolve(options["--out"]);
  // Never write directly into the bundled reference directory or overwrite a release.
  const bundled = resolve(ROOT, "src-tauri/reference").toLowerCase();
  if (
    output.toLowerCase() === bundled ||
    output.toLowerCase().startsWith(bundled + "/") ||
    output.toLowerCase().startsWith(bundled + "\\")
  )
    fail("Output cannot be the bundled catalog directory");
  const staging = join(
    dirname(output),
    `.reference-${process.pid}-${Date.now()}`,
  );
  mkdirSync(staging);
  try {
    writeFileSync(join(staging, "catalog.json"), json(candidate), {
      flag: "wx",
    });
    writeFileSync(join(staging, "provenance.json"), json(record), {
      flag: "wx",
    });
    // mkdir is an exclusive reservation: an existing output is never replaced.
    mkdirSync(output);
    renameSync(join(staging, "catalog.json"), join(output, "catalog.json"));
    renameSync(
      join(staging, "provenance.json"),
      join(output, "provenance.json"),
    );
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
  console.log(
    `Validated candidate: ${output}\nSHA-256: ${record.outputSha256}`,
  );
}

if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
