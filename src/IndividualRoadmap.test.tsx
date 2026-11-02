import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { IndividualRoadmap } from "./IndividualRoadmap";
import {
  strategyNative,
  type RoadmapView,
  type StrategyReport,
} from "./strategy-native";
vi.mock("./strategy-native", () => ({
  strategyNative: { roadmap: vi.fn(), generate: vi.fn(), progress: vi.fn() },
}));
const id = "00000000-0000-4000-8000-000000000001";
const report: StrategyReport = {
  pokemonId: id,
  displayName: "Fiction",
  rulesetVersion: "collection-1",
  observationRevision: "1",
  datasetId: "fictional",
  datasetVersion: "one",
  mechanicsVersion: null,
  mechanicsSha256: null,
  protected: true,
  uniqueForm: null,
  duplicateCount: 0,
  duplicateIds: [],
  leagues: [],
  recommendations: [],
  limitations: [],
  steps: [
    {
      id: "review-identity",
      action: "review",
      required: true,
      prerequisites: [],
      rationale: "Confirm species and form",
      cost: null,
      executable: true,
    },
    {
      id: "wait",
      action: "wait",
      required: false,
      prerequisites: [],
      rationale: "Wait for verified mechanics",
      cost: null,
      executable: true,
    },
  ],
};
const view: RoadmapView = {
  context: "a".repeat(64),
  currentContext: "a".repeat(64),
  revision: "3",
  report,
  previous: null,
  progress: {},
  stale: false,
  generatedAt: "2026-11-01 12:00:00",
};
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(strategyNative.roadmap).mockResolvedValue(view);
  vi.mocked(strategyNative.progress).mockResolvedValue({
    ...view,
    revision: "4",
    progress: { "review-identity": "review_requested" },
  });
});
afterEach(cleanup);
it("routes completion reports to selected-UUID confirmation without claiming a game action", async () => {
  const open = vi.fn();
  render(<IndividualRoadmap id={id} onOpenIndividual={open} />);
  const rationale = await screen.findByText("Confirm species and form");
  const step = rationale.closest("li")!;
  fireEvent.click(
    within(step).getByRole("button", { name: "I completed this" }),
  );
  await screen.findByText(/Progress: review_requested/);
  expect(strategyNative.progress).toHaveBeenCalledExactlyOnceWith(
    id,
    view.context,
    "3",
    "review-identity",
    "review_requested",
  );
  expect(open).toHaveBeenCalledExactlyOnceWith(id);
  expect(strategyNative.generate).not.toHaveBeenCalled();
});
it("withholds stale progress and recalculates using the current context", async () => {
  vi.mocked(strategyNative.roadmap).mockResolvedValue({
    ...view,
    stale: true,
    currentContext: "b".repeat(64),
  });
  vi.mocked(strategyNative.generate).mockResolvedValue({
    ...view,
    context: "b".repeat(64),
    currentContext: "b".repeat(64),
    revision: "4",
  });
  render(<IndividualRoadmap id={id} onOpenIndividual={vi.fn()} />);
  await screen.findByText(/saved roadmap is stale/);
  for (const button of screen.getAllByRole("button", {
    name: "I completed this",
  }))
    expect(button).toBeDisabled();
  for (const button of screen.getAllByRole("button", { name: "Postpone" }))
    expect(button).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Recalculate roadmap" }));
  await screen.findByText(/progress revision 4/);
  expect(strategyNative.generate).toHaveBeenCalledExactlyOnceWith(
    id,
    "b".repeat(64),
  );
  expect(strategyNative.progress).not.toHaveBeenCalled();
});
it("retains a failed progress draft and reloads after a conflict", async () => {
  vi.mocked(strategyNative.progress).mockRejectedValueOnce({
    code: "conflict",
    message: "Roadmap changed",
  });
  render(<IndividualRoadmap id={id} onOpenIndividual={vi.fn()} />);
  await screen.findByText("Confirm species and form");
  fireEvent.click(screen.getAllByRole("button", { name: "Postpone" })[0]);
  expect(await screen.findByRole("alert")).toHaveTextContent("Roadmap changed");
  fireEvent.click(screen.getByRole("button", { name: "Reload roadmap" }));
  await screen.findByText(/progress revision 3/);
  expect(strategyNative.roadmap).toHaveBeenCalledTimes(2);
});

it("links a gym preparation option to explicit move review for the same UUID", async () => {
  const gymReport: StrategyReport = {
    ...report,
    gym: {
      state: "limited",
      rulesetVersion: "fictional-gym-1",
      summary: "Potential only",
      pokemonTypes: ["normal"],
      vulnerableTo: [],
      missing: [],
      limitations: [],
      ownedAlternativePokemonId: null,
      builds: [
        {
          attackType: "normal",
          fastMove: "Sample Tap",
          chargedMove: "Sample Slam",
          moveState: "hypothetical",
          neutralCycleDps: 12,
          referencePercentile: 50,
          comparedForms: 3,
          referenceBest: [],
          strongAgainst: ["fire"],
          resistedBy: [],
        },
      ],
    },
    steps: [
      {
        id: "gym-review-moves",
        action: "review",
        required: false,
        prerequisites: [],
        rationale: "Optionally confirm current moves before preparing for gyms",
        cost: null,
        executable: true,
      },
    ],
  };
  vi.mocked(strategyNative.roadmap).mockResolvedValue({
    ...view,
    report: gymReport,
  });
  vi.mocked(strategyNative.progress).mockResolvedValue({
    ...view,
    report: gymReport,
    revision: "4",
    progress: { "gym-review-moves": "review_requested" },
  });
  const open = vi.fn();
  render(<IndividualRoadmap id={id} onOpenIndividual={open} />);
  expect(
    await screen.findByText(
      /Gym preparation focus: normal coverage with Sample Tap/,
    ),
  ).toBeVisible();
  const step = screen
    .getByText("Optionally confirm current moves before preparing for gyms")
    .closest("li")!;
  fireEvent.click(
    within(step).getByRole("button", { name: "I completed this" }),
  );
  await screen.findByText(/Progress: review_requested/);
  expect(open).toHaveBeenCalledExactlyOnceWith(id);
  expect(strategyNative.progress).toHaveBeenCalledExactlyOnceWith(
    id,
    view.context,
    "3",
    "gym-review-moves",
    "review_requested",
  );
});
