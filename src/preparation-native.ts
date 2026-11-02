import { invoke } from "@tauri-apps/api/core";

export interface ResourceCost {
  candy: number | null;
  candyXl: number | null;
  stardust: number | null;
  items: string[];
}
export interface PreparationAction {
  kind: string;
  description: string;
  requirements: string[];
  acquisition: string[];
}
export interface PreparationPath {
  id: string;
  targetFormId: string;
  targetName: string;
  targetLevel: number | null;
  role: string;
  actions: PreparationAction[];
  cost: ResourceCost;
  benefit: string;
  blockers: string[];
  affordability: string;
  executable: boolean;
  futureRoles?: RoleAssessment[];
  servedRoles?: string[];
  projectedCp?: number | null;
  improvementPercent?: number | null;
}
export interface ResourceValues {
  candy: number | null;
  candyXl: number | null;
  stardust: number | null;
  maximumLevel: number | null;
  normalCostsConfirmed: boolean;
  selectedPathId?: string | null;
}
export interface ResourceBudget extends ResourceValues {
  revision: string;
}
export interface PreparationPlan {
  state: string;
  selectedPathId: string | null;
  summary: string;
  currentLevel: number | null;
  paths: PreparationPath[];
  missing: string[];
  limitations: string[];
  resources?: ResourceBudget;
  primaryPathId?: string | null;
  planningMode?: string;
  conflicts?: PlanConflict[];
}
export interface PlanConflict {
  pathId: string;
  role: string;
  explanation: string;
  severity: "review" | "blocked";
}
export interface RoleAssessment {
  role: string;
  state: string;
  summary: string;
  reasons: string[];
  missing: string[];
}
export const preparationNative = {
  clearRoute: (id: string, expectedContext: string) =>
    invoke<ResourceBudget>("clear_preparation_route", { id, expectedContext }),
  selectPath: (id: string, expectedContext: string, pathId: string) =>
    invoke<ResourceBudget>("select_preparation_path", {
      id,
      expectedContext,
      pathId,
    }),
  resources: (id: string) =>
    invoke<ResourceBudget>("preparation_resources", { id }),
  saveResources: (
    id: string,
    expectedRevision: string,
    values: ResourceValues,
  ) =>
    invoke<ResourceBudget>("save_preparation_resources", {
      id,
      expectedRevision,
      values,
    }),
};
