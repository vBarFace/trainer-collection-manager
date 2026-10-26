import test from "node:test";
import assert from "node:assert/strict";
import {
  readFileSync,
  mkdtempSync,
  rmSync,
  existsSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  transform,
  json,
  main,
  validateNative,
  digest,
} from "./reference-adapter.mjs";

const fixture = (name) =>
  fileURLToPath(new URL(`fixtures/reference/${name}.json`, import.meta.url));
const source = JSON.parse(readFileSync(fixture("source")));
const plan = JSON.parse(readFileSync(fixture("plan")));

test("complete generated candidate obeys the production Rust loader", () => {
  const candidate = transform(source, plan);
  validateNative(candidate);
  for (const mutate of [
    (c) => {
      c.forms.push(c.forms[0]);
    },
    (c) => {
      c.forms[0].speciesId = "missing";
    },
    (c) => {
      c.forms[0].types = ["invented-type"];
    },
    (c) => {
      c.forms[0].evolutions = [{ targetFormId: "missing", requirements: null }];
    },
    (c) => {
      c.forms[0].evolutions = [
        { targetFormId: c.forms[0].id, requirements: [] },
      ];
    },
    (c) => {
      c.forms[0].aliases = [c.forms[0].name];
    },
    (c) => {
      delete c.forms[0].baseStats;
    },
    (c) => {
      c.species[0].aliases = ["Alias", "ALIAS"];
    },
  ]) {
    const invalid = structuredClone(candidate);
    mutate(invalid);
    assert.throws(() => validateNative(invalid), /Native validation failed/);
  }
});

test("offline CLI validates before emitting immutable output and reproducible provenance", () => {
  const directory = mkdtempSync(join(tmpdir(), "tcm-reference-test-"));
  try {
    const run = (out, extra = []) =>
      main([
        "--source",
        fixture("source"),
        "--plan",
        fixture("plan"),
        "--out",
        out,
        ...extra,
      ]);
    const first = join(directory, "first");
    const second = join(directory, "second");
    run(first);
    run(second, ["--previous", join(first, "provenance.json")]);
    const bytes = readFileSync(join(first, "catalog.json"));
    assert.equal(bytes.toString(), json(transform(source, plan)));
    assert.deepEqual(bytes, readFileSync(join(second, "catalog.json")));
    const p = JSON.parse(readFileSync(join(first, "provenance.json")));
    assert.equal(p.sourceSha256, digest(readFileSync(fixture("source"))));
    assert.equal(p.planSha256, digest(readFileSync(fixture("plan"))));
    assert.equal(p.outputSha256, digest(bytes));
    assert.deepEqual(
      p,
      JSON.parse(readFileSync(join(second, "provenance.json"))),
    );
    assert.throws(() => run(first), /EEXIST/);
    assert.deepEqual(bytes, readFileSync(join(first, "catalog.json")));
    assert.throws(() =>
      main([
        "--source",
        fixture("plan"),
        "--plan",
        fixture("plan"),
        "--out",
        join(directory, "invalid"),
      ]),
    );
    assert.equal(existsSync(join(directory, "invalid")), false);
    const malformed = join(directory, "malformed.json");
    writeFileSync(malformed, "{broken");
    assert.throws(
      () =>
        main([
          "--source",
          malformed,
          "--plan",
          fixture("plan"),
          "--out",
          join(directory, "parse-failed"),
        ]),
      SyntaxError,
    );
    assert.equal(existsSync(join(directory, "parse-failed")), false);
    assert.throws(
      () =>
        run(join(directory, "missing-history"), [
          "--previous",
          join(directory, "absent.json"),
        ]),
      /ENOENT/,
    );
    assert.throws(() =>
      main(["--source", fixture("source"), "--source", fixture("source")]),
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
