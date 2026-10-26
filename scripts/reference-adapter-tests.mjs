import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { transform, provenance, json, digest } from "./reference-adapter.mjs";

const load = (name) =>
  JSON.parse(
    readFileSync(new URL(`fixtures/reference/${name}.json`, import.meta.url)),
  );
const fixtures = () => ({ source: load("source"), plan: load("plan") });
const record = (source, plan, previous = null) =>
  provenance(transform(source, plan), json(source), json(plan), previous);

test("deterministic source/mapping/key/alias ordering and exact digests", () => {
  const { source, plan } = fixtures();
  const first = transform(source, plan);
  const p = record(source, plan);
  assert.equal(p.sourceSha256, digest(json(source)));
  assert.equal(p.planSha256, digest(json(plan)));
  assert.equal(p.outputSha256, digest(json(first)));
  assert.equal(p.upstreamRevision, plan.dataset.upstreamRevision);
  assert.equal(p.sourceUrl, plan.dataset.sourceUrl);
  assert.equal(p.retrievedAt, "2026-10-26");
  source.reverse();
  plan.species.reverse();
  plan.forms.reverse();
  plan.species.forEach((s) => s.aliases.reverse());
  plan.dataset = Object.fromEntries(Object.entries(plan.dataset).reverse());
  assert.equal(json(transform(source, plan)), json(first));
  assert.equal(record(source, plan).outputSha256, p.outputSha256);
  assert.notEqual(record(source, plan).sourceSha256, p.sourceSha256);
  assert.deepEqual(record(source, plan), record(source, plan));
});

test("GO stats/types, explicit forms and unknown/empty evolution groups", () => {
  const { source, plan } = fixtures();
  const c = transform(source, plan);
  const standard = c.forms.find((f) => f.id === "emberwit-standard");
  assert.deepEqual(standard.baseStats, {
    attack: 123,
    defense: 144,
    stamina: 155,
  });
  assert.deepEqual(standard.types, ["fire"]);
  assert.deepEqual(standard.evolutions, [
    { targetFormId: "cindervane-standard", requirements: null },
  ]);
  assert.equal(c.forms.length, 4); // Shadow entry is outside the selected subset.
  const coastal = c.forms.find((f) => f.id === "emberwit-coastal");
  assert.equal(coastal.speciesId, "emberwit");
  assert.deepEqual(coastal.types, ["water", "flying"]);
  assert.equal(coastal.baseStats, null);
  assert.equal(coastal.evolutions, null);
  const last = c.forms.find((f) => f.id === "cindervane-standard");
  assert.equal(last.types, null);
  assert.deepEqual(last.evolutions, []);
  plan.forms[0].evolutions = null;
  assert.equal(
    transform(source, plan).forms.find((f) => f.id === "emberwit-standard")
      .evolutions,
    null,
  ); // family is never an evolution assertion
  plan.forms[0].evolutions = {
    evidenceUrl: "https://example.org/fiction",
    edges: [{ targetFormId: "cindervane-standard", requirements: [] }],
  };
  assert.deepEqual(
    transform(source, plan).forms.find((f) => f.id === "emberwit-standard")
      .evolutions[0].requirements,
    [],
  );
});

test("selected upstream schema drift, invalid stats/types and duplicate input fail", () => {
  const mutations = [
    (s) => {
      delete s[0].baseStats;
    },
    (s) => {
      delete s[0].types;
    },
    (s) => {
      s[0].newSchemaField = true;
    },
    (s) => {
      s[0].baseStats.atk = "123";
    },
    (s) => {
      delete s[0].baseStats.hp;
    },
    (s) => {
      s[0].baseStats.hp = 0;
    },
    (s) => {
      s[0].baseStats.stamina = 155;
    },
    (s) => {
      s[0].types = ["unknown"];
    },
    (s) => {
      s[0].types = ["none"];
    },
    (s) => {
      s[0].types = ["fire", "fire"];
    },
    (s) => {
      s[0].types = [];
    },
    (s) => {
      s[0].tags = null;
    },
    (s) => {
      s.push(s[0]);
    },
    (s) => {
      s[0].dex = 61003;
    },
    (s) => {
      s.shift();
    },
    (s, p) => {
      p.species.push(p.species[0]);
    },
    (s, p) => {
      p.forms.push(p.forms[0]);
    },
    (s, p) => {
      delete p.forms[0].evolutions;
    },
    (s, p) => {
      p.forms[0].kind = "unknown";
    },
    (s, p) => {
      p.forms[0].evidenceUrl = "";
    },
    (s, p) => {
      p.forms[2].kind = "standard";
    },
    (s, p) => {
      p.forms[2].evolutions = {
        evidenceUrl: "https://example.org/fiction",
        edges: [],
      };
    },
  ];
  for (const mutate of mutations) {
    const { source, plan } = fixtures();
    mutate(source, plan);
    assert.throws(() => transform(source, plan));
  }
});

test("individual attributes cannot be mapped into forms", () => {
  for (const attribute of ["shadow", "purified", "shiny", "lucky"]) {
    const { source, plan } = fixtures();
    source[0].tags = [attribute];
    assert.throws(() => transform(source, plan), /Individual attributes/);
    source[0].tags = [];
    source.pop(); // Remove the separately supplied Shadow entry before renaming.
    source[0].speciesId += `_${attribute}`;
    plan.forms[0].upstreamId = source[0].speciesId;
    assert.throws(() => transform(source, plan), /Individual attributes/);
  }
});

test("rights, pinned revision, and provenance fail closed", () => {
  for (const mutate of [
    (p) => {
      p.rights.status = "blocked";
    },
    (p) => {
      p.rights.status = "reviewed";
      p.dataset.upstreamRevision = "master";
    },
    (p) => {
      p.rights.status = "reviewed";
      p.dataset.upstreamRevision = "a".repeat(40);
    },
    (p) => {
      p.dataset.datasetId = "production";
    },
    (p) => {
      p.dataset.upstreamRevision = "latest";
    },
    (p) => {
      p.dataset.retrievedAt = "2026-02-30";
    },
    (p) => {
      p.dataset.rightsEvidenceUrl = "https://example.org/other";
    },
    (p) => {
      delete p.dataset.notice;
    },
  ]) {
    const { source, plan } = fixtures();
    mutate(plan);
    assert.throws(() => transform(source, plan));
  }
});

test("immutable versions, compatible renames/additions and tombstones across releases", () => {
  const { source, plan } = fixtures();
  const first = record(source, plan);
  assert.equal(record(source, plan, first).outputSha256, first.outputSha256);
  plan.species[0].name = "Renamed fictional species";
  assert.throws(() => record(source, plan, first), /version\/content mismatch/);
  plan.dataset.version = "fiction-2";
  plan.species[0].aliases = [];
  plan.forms[0].name = "Renamed form";
  source[0].baseStats = null;
  const second = record(source, plan, first);
  assert.equal(second.identityLedger.versions.length, 2);
  plan.forms = plan.forms.filter((f) => f.id !== "emberwit-coastal");
  plan.dataset.version = "fiction-3";
  const third = record(source, plan, second);
  assert.equal(third.identityLedger.forms.length, 4);
  const coastal = load("plan").forms[1];
  coastal.id = "recycled-coastal";
  plan.forms.push(coastal);
  plan.dataset.version = "fiction-4";
  assert.throws(() => record(source, plan, third), /Form identity conflict/);
  coastal.id = "emberwit-coastal";
  const fourth = record(source, plan, third);
  assert.equal(fourth.identityLedger.forms.length, 4);
  source.push({
    dex: 61003,
    speciesName: "Mistquill",
    speciesId: "mistquill",
    baseStats: null,
    types: null,
    tags: [],
  });
  plan.species.push({
    id: "mistquill",
    dexNumber: 61003,
    name: "Mistquill",
    aliases: [],
    evidenceUrl: "https://example.org/fiction",
  });
  plan.forms.push({
    ...coastal,
    id: "mistquill-standard",
    upstreamId: "mistquill",
    speciesId: "mistquill",
    kind: "standard",
  });
  plan.dataset.version = "fiction-5";
  assert.equal(record(source, plan, fourth).identityLedger.species.length, 3);
});

test("identity conflicts and invalid previous ledgers require explicit review", () => {
  for (const mutate of [
    (s, p) => {
      s.filter((r) => r.dex === 61001).forEach((r) => {
        r.dex = 61003;
      });
      p.species[0].dexNumber = 61003;
    },
    (s, p) => {
      p.species[0].id = "reused";
      p.forms
        .filter((f) => f.speciesId === "emberwit")
        .forEach((f) => {
          f.speciesId = "reused";
        });
    },
    (s, p) => {
      p.forms[0].id = "replacement-form";
    },
    (s, p) => {
      p.forms[0].kind = "other";
    },
  ]) {
    const { source, plan } = fixtures();
    const first = record(source, plan);
    mutate(source, plan);
    plan.dataset.version = "fiction-2";
    assert.throws(() => record(source, plan, first), /identity conflict/);
  }
  const { source, plan } = fixtures();
  const p = record(source, plan);
  p.identityLedger.species.push(p.identityLedger.species[0]);
  assert.throws(() => record(source, plan, p), /Duplicate ledger/);
});

test("Mega targets and corrupted previous provenance are rejected", () => {
  const { source, plan } = fixtures();
  const first = record(source, plan);
  plan.forms[0].evolutions.edges[0].targetFormId = "emberwit-mega";
  assert.throws(() => transform(source, plan), /Mega transformations/);
  for (const field of ["sourceSha256", "planSha256", "outputSha256"]) {
    const previous = structuredClone(first);
    previous[field] = "not-a-digest";
    assert.throws(
      () => record(source, load("plan"), previous),
      /provenance digest/,
    );
  }
  first.outputSha256 = "a".repeat(64);
  assert.throws(
    () => record(source, load("plan"), first),
    /missing from version ledger/,
  );
});
