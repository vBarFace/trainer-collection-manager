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
}
export interface RoleAssessment {
  role: string;
  state: string;
  summary: string;
  reasons: string[];
  missing: string[];
}
export const preparationNative = {
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
