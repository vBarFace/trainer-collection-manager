import { invoke } from "@tauri-apps/api/core";
import type { RoadmapView, StrategyReport } from "./strategy-native";

export interface SuggestedUse {
  id: string;
  tag: string;
  state: string;
  targetFormId: string | null;
  targetName: string | null;
  explanation: string;
  evidence: string[];
  missing: string[];
}

export interface IndividualDecision {
  summary: string;
  suggestions: SuggestedUse[];
  missing: string[];
}

export interface IndividualDecisionResult {
  context: string;
  report: StrategyReport;
  roadmap: RoadmapView;
  notice: string | null;
}

export const individualDecisionNative = {
  evaluate: (id: string) =>
    invoke<IndividualDecisionResult>("individual_decision", { id }),
};
