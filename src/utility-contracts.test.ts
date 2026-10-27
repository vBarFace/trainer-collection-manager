import { describe, expect, expectTypeOf, it } from "vitest";
import fixtureJson from "../tests/fixtures/utility/acceptance.json";
import {
  UTILITY_CONTRACT_VERSION,
  type ActionKind,
  type ActionReport,
  type ActionTag,
  type Affordability,
  type Assessment,
  type Condition,
  type ConditionState,
  type Constraints,
  type DatasetVersion,
  type EvidenceContext,
  type EvolutionTarget,
  type Goal,
  type ModelMode,
  type NextAction,
  type Path,
  type PathKind,
  type Provenance,
  type Readiness,
  type ResourceAmount,
  type Role,
  type RoleAssessment,
  type Step,
  type StepAction,
  type SummaryTag,
  type Target,
  type Validity,
} from "./utility-contracts";

// Test-only shape readers. Each field map must cover exactly keyof its wire DTO.
// These exercise actual shared JSON without asserting it blindly to Assessment.
// Rust owns structural invariants; no renderer parser is exposed to the app.
type Check = (value: unknown) => void;
type Fields<T> = { [K in keyof T]-?: Check };
function record(value: unknown): asserts value is Record<string, unknown> {
  expect(value).not.toBeNull();
  expect(typeof value).toBe("object");
  expect(Array.isArray(value)).toBe(false);
}
function object<T>(value: unknown, fields: Fields<T>): asserts value is T {
  record(value);
  expect(Object.keys(value).sort()).toEqual(Object.keys(fields).sort());
  for (const [key, check] of Object.entries(fields))
    (check as Check)(value[key]);
}
const string: Check = (value) => expect(typeof value).toBe("string");
const boolean: Check = (value) => expect(typeof value).toBe("boolean");
const number: Check = (value) => {
  expect(typeof value).toBe("number");
  expect(Number.isFinite(value)).toBe(true);
};
const nullable =
  (check: Check): Check =>
  (value) => {
    if (value !== null) check(value);
  };
const array =
  (check: Check): Check =>
  (value) => {
    expect(Array.isArray(value)).toBe(true);
    if (!Array.isArray(value)) throw new Error("Expected an explicit array");
    value.forEach(check);
  };
const oneOf =
  (values: readonly unknown[]): Check =>
  (value) =>
    expect(values).toContain(value);
const roles = [
  "ATTACKER",
  "GYM DEF",
  "ROCKET",
  "PVP 1500",
  "PVP 2500",
] as const satisfies readonly Role[];
const readiness = [
  "usable_now",
  "usable_with_limitations",
  "preparation_required",
  "insufficient_data",
] as const satisfies readonly Readiness[];
const validity = [
  "current",
  "blocked",
  "stale",
  "invalid",
] as const satisfies readonly Validity[];
const pathKinds = [
  "prepare",
  "use_now",
  "wait",
] as const satisfies readonly PathKind[];
const affordability = [
  "unknown",
  "within_budget",
  "over_budget",
] as const satisfies readonly Affordability[];
const conditionStates = [
  "satisfied",
  "unmet",
  "unknown",
] as const satisfies readonly ConditionState[];
const actions = [
  "evolve",
  "power_up",
  "change_moves",
] as const satisfies readonly ActionKind[];
const actionTags = [
  "EVOLVE",
  "POWER UP",
] as const satisfies readonly ActionTag[];
const integerString: Check = (value) => {
  string(value);
  expect(value).toMatch(/^(0|[1-9][0-9]*)$/);
  if (typeof value !== "string") throw new Error("Expected a decimal string");
  expect(BigInt(value)).toBeLessThanOrEqual(9223372036854775807n);
};
const resource: Check = (value) =>
  object<ResourceAmount>(value, {
    resource: string,
    amount: nullable(integerString),
  });
const constraints: Check = (value) =>
  object<Constraints>(value, {
    budget: nullable(array(resource)),
    evolutionAllowed: nullable(boolean),
  });
const dataset: Check = (value) =>
  object<DatasetVersion>(value, {
    datasetId: string,
    version: string,
    sha256: (hash) => expect(hash).toMatch(/^[a-f0-9]{64}$/),
  });
const model: Check = (value) =>
  object<ModelMode>(value, { mode: oneOf(["deterministic_only"]) });
const context: Check = (value) =>
  object<EvidenceContext>(value, {
    observationRevision: integerString,
    constraintRevision: integerString,
    rulesetVersion: string,
    datasets: nullable(array(dataset)),
    model,
  });
const provenance: Check = (value) =>
  object<Provenance>(value, { evaluated: context, current: context });
const roleAssessment: Check = (value) =>
  object<RoleAssessment>(value, {
    role: oneOf(roles),
    contextId: string,
    readiness: oneOf(readiness),
    explanation: string,
  });
const goal: Check = (value) =>
  object<Goal>(value, {
    id: string,
    role: oneOf(roles),
    contextId: string,
    description: string,
  });
const target: Check = (value) =>
  object<Target>(value, {
    minimum: number,
    maximum: number,
    unit: string,
    contextId: string,
    rationale: string,
  });
const evolutionTarget: Check = (value) =>
  object<EvolutionTarget>(value, {
    datasetId: string,
    speciesId: string,
    formId: string,
  });
const stepAction: Check = (value) => {
  record(value);
  switch (value.kind) {
    case "evolve":
      return object<Extract<StepAction, { kind: "evolve" }>>(value, {
        kind: oneOf(["evolve"]),
        target: evolutionTarget,
      });
    case "power_up":
      return object<Extract<StepAction, { kind: "power_up" }>>(value, {
        kind: oneOf(["power_up"]),
        target,
      });
    case "change_moves":
      return object<Extract<StepAction, { kind: "change_moves" }>>(value, {
        kind: oneOf(["change_moves"]),
        moves: array(string),
      });
    case "use_role":
      return object<Extract<StepAction, { kind: "use_role" }>>(value, {
        kind: oneOf(["use_role"]),
        role: oneOf(roles),
      });
    case "collect_evidence":
      return object<Extract<StepAction, { kind: "collect_evidence" }>>(value, {
        kind: oneOf(["collect_evidence"]),
        fields: array(string),
      });
    default:
      throw new Error("Unknown step action kind");
  }
};
const condition: Check = (value) =>
  object<Condition>(value, {
    description: string,
    state: oneOf(conditionStates),
  });
const step: Check = (value) =>
  object<Step>(value, {
    id: string,
    required: boolean,
    action: stepAction,
    prerequisites: array(string),
    conditions: nullable(array(condition)),
    explanation: string,
  });
const path: Check = (value) =>
  object<Path>(value, {
    id: string,
    pokemonId: string,
    goalId: string,
    kind: oneOf(pathKinds),
    steps: array(step),
    estimatedCosts: nullable(array(resource)),
    affordability: oneOf(affordability),
    expectedBenefit: string,
    limitations: array(string),
  });
const nextAction: Check = (value) =>
  object<NextAction>(value, {
    pathId: string,
    stepId: string,
    kind: oneOf(actions),
  });
const summary: Check = (value) => {
  record(value);
  switch (value.kind) {
    case "role":
      return object<Extract<SummaryTag, { kind: "role" }>>(value, {
        kind: oneOf(["role"]),
        tag: oneOf(roles),
      });
    case "action":
      return object<Extract<SummaryTag, { kind: "action" }>>(value, {
        kind: oneOf(["action"]),
        tag: oneOf(actionTags),
      });
    default:
      throw new Error("Unknown summary tag kind");
  }
};
const report: Check = (value) =>
  object<ActionReport>(value, {
    pathId: string,
    stepId: string,
    observationRevision: integerString,
    constraintRevision: integerString,
  });
function assessment(value: unknown): asserts value is Assessment {
  object<Assessment>(value, {
    contractVersion: oneOf([UTILITY_CONTRACT_VERSION]),
    pokemonId: (uuid) =>
      expect(uuid).toMatch(
        /^[a-f0-9]{8}-[a-f0-9]{4}-4[a-f0-9]{3}-[89ab][a-f0-9]{3}-[a-f0-9]{12}$/,
      ),
    goal,
    currentUsefulness: string,
    currentRoles: nullable(array(roleAssessment)),
    potentialRoles: nullable(array(roleAssessment)),
    unsupportedRoles: array(oneOf(roles)),
    validity: oneOf(validity),
    validityReasons: array(string),
    suggestedAction: nullable(nextAction),
    suggestedSummaryTag: nullable(summary),
    selectedPathId: string,
    paths: array(path),
    constraints,
    requiredInputs: array(string),
    missingEvidence: array(string),
    assumptions: array(string),
    limitations: array(string),
    provenance,
    actionReports: array(report),
  });
}
interface FixtureSet {
  notice: string;
  cases: { id: string; assessment: Assessment }[];
}
function fixtures(value: unknown): asserts value is FixtureSet {
  object<FixtureSet>(value, {
    notice: string,
    cases: array((entry) =>
      object<FixtureSet["cases"][number]>(entry, { id: string, assessment }),
    ),
  });
}
fixtures(fixtureJson);
const cases: FixtureSet = fixtureJson;
const get = (id: string): Assessment => {
  const found = cases.cases.find((item) => item.id === id);
  if (!found) throw new Error(`Missing shared case: ${id}`);
  return found.assessment;
};

describe("shared fictional utility wire contracts", () => {
  it("covers the complete TypeScript vocabulary without inventing default tags", () => {
    expectTypeOf<Role>().toEqualTypeOf<(typeof roles)[number]>();
    expectTypeOf<Readiness>().toEqualTypeOf<(typeof readiness)[number]>();
    expectTypeOf<Validity>().toEqualTypeOf<(typeof validity)[number]>();
    expectTypeOf<PathKind>().toEqualTypeOf<(typeof pathKinds)[number]>();
    expectTypeOf<Affordability>().toEqualTypeOf<
      (typeof affordability)[number]
    >();
    expectTypeOf<ConditionState>().toEqualTypeOf<
      (typeof conditionStates)[number]
    >();
    expectTypeOf<ActionKind>().toEqualTypeOf<(typeof actions)[number]>();
    expectTypeOf<ActionTag>().toEqualTypeOf<(typeof actionTags)[number]>();
    expect(cases.notice).toMatch(/fictional/i);
    expect(cases.cases).toHaveLength(9);
  });

  it.each(cases.cases)("reads and losslessly round-trips $id", (entry) => {
    assessment(entry.assessment);
    const roundTrip: unknown = JSON.parse(JSON.stringify(entry.assessment));
    assessment(roundTrip);
    expect(roundTrip).toEqual(entry.assessment);
    expect(roundTrip.provenance.evaluated.model).toEqual({
      mode: "deterministic_only",
    });
  });

  it("distinguishes unknown, empty, zero and exact large integer quantities", () => {
    const missing = get("missing-evidence");
    const zero = get("zero-budget");
    const unknown = get("unknown-budget");
    expect(missing.currentRoles).toBeNull();
    expect(get("evolution-first").currentRoles).toEqual([]);
    expect(missing.provenance.evaluated.datasets).toBeNull();
    expect(zero.provenance.evaluated.datasets).toEqual([]);
    expect(missing.paths[0].steps[0].conditions).toBeNull();
    expect(zero.paths[0].steps[0].conditions).toEqual([]);
    expect(zero.constraints.budget?.[0].amount).toBe("0");
    expect(unknown.constraints.budget?.[0].amount).toBeNull();
    expect(missing.constraints.budget).toBeNull();
    expect(unknown.paths[0].estimatedCosts?.[0].amount).toBeNull();
    expect(zero.paths[0].estimatedCosts).toEqual([]);
    expect(get("required-move-change").constraints.budget?.[0].amount).toBe(
      "9223372036854775807",
    );
    expect(
      get("required-move-change").paths[0].estimatedCosts?.[0].amount,
    ).toBe("9007199254740993");
    expect(missing.provenance.evaluated.observationRevision).toBe(
      "9007199254740993",
    );
  });

  it("keeps potential usefulness distinct and leaves future dependencies in place", () => {
    const evolved = get("evolution-first");
    expect(evolved.currentRoles).toEqual([]);
    expect(evolved.potentialRoles?.[0].readiness).toBe("preparation_required");
    expect(evolved.suggestedSummaryTag).toEqual({
      kind: "action",
      tag: "EVOLVE",
    });
    expect(evolved.paths[0].steps[0].prerequisites).toEqual([]);
    expect(evolved.paths[0].steps[1].prerequisites).toEqual(["prepare-evolve"]);
    expect(evolved.suggestedAction?.stepId).toBe("prepare-evolve");
  });

  it("keeps use now and optional investment costs in separate alternatives", () => {
    const now = get("usable-now-optional-investment");
    expect(now.suggestedAction).toBeNull();
    expect(now.suggestedSummaryTag).toEqual({ kind: "role", tag: "ATTACKER" });
    expect(now.selectedPathId).toBe("use-now");
    expect(now.paths[0].estimatedCosts).toEqual([]);
    expect(now.paths[1].estimatedCosts?.[0].amount).toBe("9");
    expect(now.paths[1].steps[0].required).toBe(false);
    const zero = get("zero-budget");
    expect(zero.paths.map((item) => item.affordability)).toEqual([
      "within_budget",
      "over_budget",
    ]);
  });

  it("withholds unsupported/stale suggestions while retaining reports as reports only", () => {
    for (const id of [
      "missing-evidence",
      "unknown-budget",
      "protection-conflict",
      "stale-context-report",
    ]) {
      expect(get(id).suggestedAction).toBeNull();
      expect(get(id).suggestedSummaryTag).toBeNull();
    }
    const stale = get("stale-context-report");
    expect(stale.provenance.current).not.toEqual(stale.provenance.evaluated);
    expect(stale.actionReports[0].observationRevision).toBe("9007199254740993");
    expect(stale.provenance.current.observationRevision).toBe(
      "9007199254740994",
    );
    expect(Object.keys(stale.actionReports[0]).sort()).toEqual([
      "constraintRevision",
      "observationRevision",
      "pathId",
      "stepId",
    ]);
    expect(stale.unsupportedRoles).toContain("GYM DEF");
  });

  it("represents a required move change with an action and no invented default tag", () => {
    const moves = get("required-move-change");
    expect(moves.suggestedAction?.kind).toBe("change_moves");
    expect(moves.paths[0].steps[0].required).toBe(true);
    expect(moves.paths[0].steps[0].action).toEqual({
      kind: "change_moves",
      moves: ["Paper Whisper", "Origami Pulse"],
    });
    expect(moves.suggestedSummaryTag).toBeNull();
  });

  it("rejects omitted nulls, renamed wire fields, extra fields and role/action mixing", () => {
    const raw: Record<string, unknown> = { ...get("evolution-first") };
    delete raw.suggestedAction;
    expect(() => assessment(raw)).toThrow();
    expect(() =>
      assessment({ ...get("evolution-first"), pokemon_id: "wrong-key" }),
    ).toThrow();
    expect(() =>
      assessment({ ...get("evolution-first"), assignedTags: ["EVOLVE"] }),
    ).toThrow();
    expect(() =>
      assessment({
        ...get("evolution-first"),
        suggestedSummaryTag: { kind: "role", tag: "EVOLVE" },
      }),
    ).toThrow();
    expect(() =>
      assessment({
        ...get("evolution-first"),
        suggestedSummaryTag: { kind: "action", tag: "ATTACKER" },
      }),
    ).toThrow();
    expect(() =>
      context({
        ...get("evolution-first").provenance.evaluated,
        observationRevision: 9007199254740992,
      }),
    ).toThrow();
  });
});
