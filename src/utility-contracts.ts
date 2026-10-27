/** S1 wire contracts only. Not imported by live inventory or production IPC. */
export const UTILITY_CONTRACT_VERSION = "utility-1";

export type Role = "ATTACKER" | "GYM DEF" | "ROCKET" | "PVP 1500" | "PVP 2500";
export type Readiness =
  | "usable_now"
  | "usable_with_limitations"
  | "preparation_required"
  | "insufficient_data";
export type Validity = "current" | "blocked" | "stale" | "invalid";
export type PathKind = "prepare" | "use_now" | "wait";
export type Affordability = "unknown" | "within_budget" | "over_budget";
export type ConditionState = "satisfied" | "unmet" | "unknown";
export type ActionKind = "evolve" | "power_up" | "change_moves";
export type ActionTag = "EVOLVE" | "POWER UP";
export type SummaryTag =
  | { kind: "role"; tag: Role }
  | { kind: "action"; tag: ActionTag };
export interface ModelMode {
  mode: "deterministic_only";
}

export interface Goal {
  id: string;
  role: Role;
  contextId: string;
  description: string;
}
export interface RoleAssessment {
  role: Role;
  contextId: string;
  readiness: Readiness;
  explanation: string;
}
/** Canonical decimal strings preserve the nonnegative SQLite i64 range. */
export interface ResourceAmount {
  resource: string;
  amount: string | null;
}
export interface Constraints {
  budget: ResourceAmount[] | null;
  evolutionAllowed: boolean | null;
}
export interface DatasetVersion {
  datasetId: string;
  version: string;
  sha256: string;
}
export interface EvidenceContext {
  observationRevision: string;
  constraintRevision: string;
  rulesetVersion: string;
  datasets: DatasetVersion[] | null;
  model: ModelMode;
}
export interface Provenance {
  evaluated: EvidenceContext;
  current: EvidenceContext;
}
export interface EvolutionTarget {
  datasetId: string;
  speciesId: string;
  formId: string;
}
/** Finite bounded measurements; not observed CP or validated game thresholds. */
export interface Target {
  minimum: number;
  maximum: number;
  unit: string;
  contextId: string;
  rationale: string;
}
export type StepAction =
  | { kind: "evolve"; target: EvolutionTarget }
  | { kind: "power_up"; target: Target }
  | { kind: "change_moves"; moves: string[] }
  | { kind: "use_role"; role: Role }
  | { kind: "collect_evidence"; fields: string[] };
export interface Condition {
  description: string;
  state: ConditionState;
}
export interface Step {
  id: string;
  required: boolean;
  action: StepAction;
  prerequisites: string[];
  conditions: Condition[] | null;
  explanation: string;
}
export interface Path {
  id: string;
  pokemonId: string;
  goalId: string;
  kind: PathKind;
  steps: Step[];
  estimatedCosts: ResourceAmount[] | null;
  affordability: Affordability;
  expectedBenefit: string;
  limitations: string[];
}
export interface NextAction {
  pathId: string;
  stepId: string;
  kind: ActionKind;
}
/** A report contains no verified new observations, completed flag or spending. */
export interface ActionReport {
  pathId: string;
  stepId: string;
  observationRevision: string;
  constraintRevision: string;
}
export interface Assessment {
  contractVersion: string;
  pokemonId: string;
  goal: Goal;
  currentUsefulness: string;
  currentRoles: RoleAssessment[] | null;
  potentialRoles: RoleAssessment[] | null;
  unsupportedRoles: Role[];
  validity: Validity;
  validityReasons: string[];
  suggestedAction: NextAction | null;
  suggestedSummaryTag: SummaryTag | null;
  selectedPathId: string;
  paths: Path[];
  constraints: Constraints;
  requiredInputs: string[];
  missingEvidence: string[];
  assumptions: string[];
  limitations: string[];
  provenance: Provenance;
  actionReports: ActionReport[];
}
