import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { Recommendations } from "./Recommendations";
import {
  strategyNative,
  type CollectionStrategy,
  type StrategyReport,
} from "./strategy-native";
vi.mock("./strategy-native", () => ({
  strategyNative: {
    collection: vi.fn(),
    savePreferences: vi.fn(),
    roadmap: vi.fn(),
    generate: vi.fn(),
    progress: vi.fn(),
  },
}));
const report: StrategyReport = {
  pokemonId: "00000000-0000-4000-8000-000000000001",
  displayName: "Fictional Emberwit",
  rulesetVersion: "collection-1",
  observationRevision: "7",
  datasetId: "fictional",
  datasetVersion: "one",
  mechanicsVersion: "fictional-cpm",
  mechanicsSha256: "a".repeat(64),
  protected: true,
  uniqueForm: true,
  duplicateCount: 0,
  duplicateIds: [],
  leagues: [
    {
      league: "Great League",
      cpLimit: 1500,
      state: "statistical_potential",
      explanation:
        "Original fictional calculation with a modeled ceiling of 40.",
      alternativePokemonId: "00000000-0000-4000-8000-000000000002",
      potential: {
        level: 10,
        levelCeiling: 40,
        cp: 250,
        attack: 50,
        defense: 50,
        hp: 50,
        statProduct: 125000,
        ivRank: 42,
        rankedSpreads: 4096,
      },
    },
  ],
  recommendations: [
    {
      id: "keep-protected",
      kind: "KEEP",
      priority: 0,
      explanation: "Retain because you explicitly protected this individual.",
      facts: ["User-confirmed protection"],
      missing: [],
      alternativePokemonId: null,
      executable: true,
    },
    {
      id: "wait",
      kind: "WAIT / NO ACTION",
      priority: 90,
      explanation: "Unknown costs block spending.",
      facts: [],
      missing: ["Moves and costs"],
      alternativePokemonId: null,
      executable: true,
    },
  ],
  steps: [],
  limitations: ["No battle-ranking claim"],
};
const snapshot: CollectionStrategy = {
  context: "b".repeat(64),
  reports: [report],
  preferences: { goal: "balanced", stardustBudget: null, revision: "0" },
};
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(strategyNative.collection).mockResolvedValue(snapshot);
  vi.mocked(strategyNative.roadmap).mockResolvedValue({
    context: snapshot.context,
    currentContext: snapshot.context,
    revision: "0",
    report: null,
    previous: null,
    progress: {},
    stale: false,
    generatedAt: null,
  });
});
afterEach(cleanup);
it("shows explainable facts and distinguishes limited statistics from battle rankings", async () => {
  const open = vi.fn();
  render(<Recommendations onOpenIndividual={open} />);
  await screen.findByText("User-confirmed protection");
  expect(screen.getByText(/Unknown costs block/)).toBeVisible();
  fireEvent.click(
    screen.getByRole("button", {
      name: "Statistics and assessment for Fictional Emberwit",
    }),
  );
  expect(screen.getByText(/IV rank 42 \/ 4096/)).toHaveTextContent(
    "Modeled ceiling 40",
  );
  expect(screen.getByText(/does not rank battle performance/)).toBeVisible();
  fireEvent.click(
    screen.getByRole("button", {
      name: "Inspect owned alternative for Great League",
    }),
  );
  expect(open).toHaveBeenCalledExactlyOnceWith(
    "00000000-0000-4000-8000-000000000002",
  );
});
it("preserves unknown versus zero budget and saves global preferences explicitly", async () => {
  render(<Recommendations onOpenIndividual={vi.fn()} />);
  await screen.findByLabelText("Confirmed available Stardust budget");
  expect(
    screen.getByLabelText("Confirmed available Stardust budget"),
  ).toHaveValue("");
  fireEvent.change(
    screen.getByLabelText("Confirmed available Stardust budget"),
    { target: { value: "0" } },
  );
  fireEvent.change(screen.getByLabelText("Collection goal"), {
    target: { value: "great" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Confirm goals and budget" }),
  );
  expect(strategyNative.savePreferences).toHaveBeenCalledExactlyOnceWith(
    "0",
    "great",
    0,
  );
});
it("offers retry on native failure without inventing recommendations", async () => {
  vi.mocked(strategyNative.collection).mockRejectedValueOnce({
    code: "storage",
    message: "Reference unavailable",
  });
  render(<Recommendations onOpenIndividual={vi.fn()} />);
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Reference unavailable",
  );
  expect(
    screen.queryByText("User-confirmed protection"),
  ).not.toBeInTheDocument();
  fireEvent.click(
    screen.getByRole("button", { name: "Recalculate collection" }),
  );
  await screen.findByText("User-confirmed protection");
});
