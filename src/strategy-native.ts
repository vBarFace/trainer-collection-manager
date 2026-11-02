import { invoke } from "@tauri-apps/api/core";
import type { GymAssessment } from "./gym-native";
import type { PreparationPlan, RoleAssessment } from "./preparation-native";

export interface StrategyRecommendation {
  id: string;
  kind: string;
  priority: number;
  explanation: string;
  facts: string[];
  missing: string[];
  alternativePokemonId: string | null;
  executable: boolean;
}
export interface StrategyStep {
  id: string;
  action: string;
  required: boolean;
  prerequisites: string[];
  rationale: string;
  cost: string | null;
  executable: boolean;
}
export interface LeagueAssessment {
  league: string;
  cpLimit: number | null;
  state: string;
  explanation: string;
  alternativePokemonId: string | null;
  potential: null | {
    level: number;
    levelCeiling: number;
    cp: number;
    attack: number;
    defense: number;
    hp: number;
    statProduct: number;
    ivRank: number;
    rankedSpreads: number;
  };
}
export interface StrategyReport {
  pokemonId: string;
  displayName: string;
  rulesetVersion: string;
  observationRevision: string;
  datasetId: string;
  datasetVersion: string;
  mechanicsVersion: string | null;
  mechanicsSha256: string | null;
  protected: boolean;
  uniqueForm: boolean | null;
  duplicateCount: number;
  duplicateIds: string[];
  leagues: LeagueAssessment[];
  gym?: GymAssessment | null;
  preparation?: PreparationPlan | null;
  roles?: RoleAssessment[];
  recommendations: StrategyRecommendation[];
  steps: StrategyStep[];
  limitations: string[];
}
export interface GoalContext {
  goal: string;
  stardustBudget: number | null;
  revision: string;
}
export interface CollectionStrategy {
  context: string;
  reports: StrategyReport[];
  preferences: GoalContext;
}
export interface RoadmapView {
  context: string;
  currentContext: string;
  revision: string;
  report: StrategyReport | null;
  previous: {
    context: string;
    report: StrategyReport;
    progress: Record<string, string>;
    generatedAt: string;
  } | null;
  progress: Record<string, string>;
  stale: boolean;
  generatedAt: string | null;
}
export const strategyNative = {
  collection: () => invoke<CollectionStrategy>("collection_strategy"),
  roadmap: (id: string) => invoke<RoadmapView>("pokemon_roadmap", { id }),
  generate: (id: string, expectedContext: string) =>
    invoke<RoadmapView>("generate_roadmap", { id, expectedContext }),
  progress: (
    id: string,
    expectedContext: string,
    expectedRevision: string,
    stepId: string,
    status: string,
  ) =>
    invoke<RoadmapView>("update_roadmap_progress", {
      id,
      expectedContext,
      expectedRevision,
      stepId,
      status,
    }),
  savePreferences: (
    expectedRevision: string,
    goal: string,
    stardustBudget: number | null,
  ) =>
    invoke<GoalContext>("save_strategy_preferences", {
      expectedRevision,
      goal,
      stardustBudget,
    }),
};
