import { invoke } from "@tauri-apps/api/core";

export interface AppraisalIvs {
  attack: number;
  defense: number;
  hp: number;
}
export interface Tag {
  id: string;
  name: string;
  scope: "standard" | "local";
}
export interface InventoryRecord {
  id: string;
  displayName: string;
  cp: string | null;
  ivs: AppraisalIvs | null;
  tags: Tag[];
  evaluation: Evaluation;
}
export type EvaluationState = "pending" | "evaluated" | "needs_data" | "stale";
export const evaluationLabels: Record<EvaluationState, string> = {
  pending: "Pending",
  evaluated: "Evaluated",
  needs_data: "Needs data",
  stale: "Stale",
};
export type IvField = "attack" | "defense" | "hp";
export type StandardClassification =
  | "EVOLVE"
  | "POWER UP"
  | "ATTACKER"
  | "GYM DEF"
  | "ROCKET"
  | "PVP 1500"
  | "PVP 2500"
  | "TRANSFER"
  | "SAVE/SPECIAL";
export interface ObservedFact {
  field: IvField;
  value: number;
}
export interface Recommendation {
  code: string;
  proposedClassification: StandardClassification | null;
  explanation: string;
  observedFacts: ObservedFact[];
  rulesetVersion: string;
  limitations: string[];
  missingData: IvField[];
}
export interface AppraisalResult {
  ivs: AppraisalIvs | null;
  total: number | null;
  appraisalPercentage: number | null;
  perfect: boolean | null;
  findings: {
    code: "perfect_appraisal";
    explanation: string;
    observedFacts: ObservedFact[];
  }[];
  recommendations: Recommendation[];
  missingData: IvField[];
  limitations: string[];
}
export interface Evaluation {
  scope: "appraisal";
  state: EvaluationState;
  currentRulesetVersion: string;
  rulesetVersion: string | null;
  evaluatedAt: string | null;
  result: AppraisalResult | null;
}
export interface InventorySnapshot {
  records: InventoryRecord[];
  tags: Tag[];
}
export interface ScreenshotPreview {
  png: number[];
  width: number;
  height: number;
  extraction: ScreenshotExtraction;
  textRegions: { displayName: number[] | null; cp: number[] | null };
}
export interface ExtractedField<T> {
  value: T | null;
  confidence: "none" | "tentative";
  method: "unavailable" | "appraisal_geometry_v1" | "tesseract_lstm";
  engineConfidence?: number;
  state: "detected" | "unresolved" | "unavailable";
}
export interface ScreenshotExtraction {
  displayName: ExtractedField<string>;
  cp: ExtractedField<string>;
  attack: ExtractedField<number>;
  defense: ExtractedField<number>;
  hp: ExtractedField<number>;
  issues: { code: string; message: string }[];
}
export interface ImportInput {
  displayName: string;
  cp: string | null;
  ivs: AppraisalIvs | null;
  tagIds: string[];
}

export const native = {
  snapshot: () => invoke<InventorySnapshot>("inventory_snapshot"),
  evaluate: (id: string) => invoke<InventoryRecord>("evaluate_pokemon", { id }),
  correctIvs: (
    id: string,
    expectedIvs: AppraisalIvs | null,
    ivs: AppraisalIvs | null,
  ) =>
    invoke<InventoryRecord>("correct_pokemon_ivs", {
      id,
      input: { expectedIvs, ivs },
    }),
  selectScreenshot: () => invoke<ScreenshotPreview | null>("select_screenshot"),
  confirmImport: (input: ImportInput) =>
    invoke<InventoryRecord>("confirm_inventory_import", { input }),
};

export function nativeErrorMessage(error: unknown, fallback: string): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    typeof error.code === "string" &&
    "message" in error &&
    typeof error.message === "string"
  )
    return error.message;
  return fallback;
}
