import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  IndividualDecisionPanel,
  IndividualDecisionView,
} from "./IndividualDecisionView";
import {
  individualDecisionNative,
  type IndividualDecisionResult,
} from "./decision-native";
import type { RoadmapView } from "./strategy-native";

vi.mock("./decision-native", () => ({
  individualDecisionNative: { evaluate: vi.fn() },
}));
vi.mock("./IndividualRoadmap", () => ({
  IndividualRoadmap: ({
    id,
    initialView,
    onReassess,
  }: {
    id: string;
    initialView?: RoadmapView;
    onReassess?: () => void;
  }) => (
    <div>
      <p>
        Roadmap individual {id}, revision {initialView?.revision}
      </p>
      <button type="button" onClick={onReassess}>
        Refresh after allocation confirmation
      </button>
    </div>
  ),
}));
const id = "00000000-0000-4000-8000-000000000001";
function result(
  revision = "1",
  summary = "Several independently supported uses",
): IndividualDecisionResult {
  const report: IndividualDecisionResult["report"] = {
    pokemonId: id,
    displayName: "Fictional Spark",
    rulesetVersion: "fictional-decision-1",
    observationRevision: revision,
    datasetId: "fictional",
    datasetVersion: "one",
    mechanicsVersion: null,
    mechanicsSha256: null,
    protected: false,
    uniqueForm: true,
    duplicateCount: 0,
    duplicateIds: [],
    leagues: [],
    recommendations: [],
    steps: [],
    limitations: [],
    decision: {
      summary,
      suggestions: [
        {
          id: "attacker",
          tag: "ATTACKER",
          state: "supported_current",
          targetFormId: "fictional-spark",
          targetName: "Fictional Spark",
          explanation:
            "Current observed set supports the stated attack coverage.",
          evidence: ["Declared external comparison"],
          missing: [],
        },
        {
          id: "league",
          tag: "PVP 2500",
          state: "needs_evidence",
          targetFormId: "fictional-grown",
          targetName: "Fictional Grown",
          explanation: "A future league option needs matchup evidence.",
          evidence: [],
          missing: ["Competitive matchups"],
        },
      ],
      missing: [],
    },
  };
  return {
    context: `context-${revision}`,
    report,
    notice: null,
    roadmap: {
      context: `context-${revision}`,
      currentContext: `context-${revision}`,
      revision,
      report,
      previous: null,
      progress: {},
      stale: false,
      generatedAt: "2026-11-02 12:00:00",
    },
  };
}
beforeEach(() => vi.clearAllMocks());
afterEach(cleanup);

it("separates qualified suggestions from unresolved uses without requiring a goal or applying tags", () => {
  render(<IndividualDecisionView decision={result().report.decision!} />);
  expect(
    screen.getByRole("heading", { name: "ATTACKER · Current use supported" }),
  ).toBeVisible();
  const review = screen
    .getByText("Other uses under review")
    .closest("details")!;
  expect(review).not.toHaveAttribute("open");
  expect(
    screen.getByText("Still needed: Competitive matchups."),
  ).not.toBeVisible();
  review.open = true;
  expect(
    screen.getByRole("heading", { name: "PVP 2500 · More evidence needed" }),
  ).toBeVisible();
  expect(screen.getByText("Evaluated form: Fictional Grown.")).toBeVisible();
  expect(screen.getByText("Still needed: Competitive matchups.")).toBeVisible();
  expect(screen.queryByRole("combobox")).not.toBeInTheDocument();
  expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
});

it("opens unresolved uses by default when only collection value is established", () => {
  const decision = result().report.decision!;
  decision.suggestions[0] = {
    ...decision.suggestions[0],
    tag: "KEEP",
    state: "collection_value",
    explanation: "Preserve collection value.",
  };
  decision.suggestions.push({
    id: "transfer",
    tag: "TRANSFER",
    state: "retention_review",
    targetFormId: null,
    targetName: null,
    explanation: "Keep while future role value is unresolved.",
    evidence: [],
    missing: ["Resolved future value"],
  });
  render(<IndividualDecisionView decision={decision} />);
  expect(
    screen.getByText("Other uses under review").closest("details"),
  ).toHaveAttribute("open");
  const suggestions = screen.getByRole("list", {
    name: "Suggested roles and retention",
  });
  expect(
    within(suggestions).getByRole("heading", {
      name: "KEEP · Keep for collection",
    }),
  ).toBeVisible();
  expect(within(suggestions).queryByText(/TRANSFER/)).not.toBeInTheDocument();
  const review = screen.getByRole("list", {
    name: "Uses requiring further review",
  });
  expect(
    within(review).getByRole("heading", {
      name: "PVP 2500 · More evidence needed",
    }),
  ).toBeVisible();
  expect(
    within(review).getByRole("heading", {
      name: "TRANSFER · Keep pending retention review",
    }),
  ).toBeVisible();
  expect(
    screen.getByText("Still needed: Resolved future value."),
  ).toBeVisible();
});

it("loads the automatic decision and returned roadmap for the same individual", async () => {
  vi.mocked(individualDecisionNative.evaluate).mockResolvedValue(result());
  render(
    <IndividualDecisionPanel
      id={id}
      observationRevision="1"
      refreshToken={0}
      onOpenIndividual={vi.fn()}
    />,
  );
  await screen.findByText("Several independently supported uses");
  expect(individualDecisionNative.evaluate).toHaveBeenCalledExactlyOnceWith(id);
  expect(
    screen.getByText(`Roadmap individual ${id}, revision 1`),
  ).toBeVisible();
});

it("automatically reevaluates changed observations and history-only action tokens", async () => {
  vi.mocked(individualDecisionNative.evaluate)
    .mockResolvedValueOnce(result())
    .mockResolvedValueOnce(result("2", "Next evolution stage assessed"))
    .mockResolvedValueOnce(result("3", "History-only Power Up reviewed"));
  const view = render(
    <IndividualDecisionPanel
      id={id}
      observationRevision="1"
      refreshToken={0}
      onOpenIndividual={vi.fn()}
    />,
  );
  await screen.findByText("Several independently supported uses");
  view.rerender(
    <IndividualDecisionPanel
      id={id}
      observationRevision="2"
      refreshToken={0}
      onOpenIndividual={vi.fn()}
    />,
  );
  await screen.findByText("Next evolution stage assessed");
  expect(
    screen.getByText(`Roadmap individual ${id}, revision 2`),
  ).toBeVisible();
  view.rerender(
    <IndividualDecisionPanel
      id={id}
      observationRevision="2"
      refreshToken={1}
      onOpenIndividual={vi.fn()}
    />,
  );
  await screen.findByText("History-only Power Up reviewed");
  expect(vi.mocked(individualDecisionNative.evaluate).mock.calls).toEqual([
    [id],
    [id],
    [id],
  ]);
});

it("keeps assessment failure separate from already confirmed Pokémon data and supports retry", async () => {
  vi.mocked(individualDecisionNative.evaluate)
    .mockRejectedValueOnce(new Error("Model unavailable"))
    .mockResolvedValueOnce(result());
  render(
    <IndividualDecisionPanel
      id={id}
      observationRevision="1"
      refreshToken={0}
      onOpenIndividual={vi.fn()}
    />,
  );
  await screen.findByText(/Recommendation refresh unavailable/);
  expect(screen.getByRole("status")).toHaveTextContent(
    "Your confirmed Pokémon data is saved",
  );
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  fireEvent.click(
    screen.getByRole("button", { name: "Retry individual assessment" }),
  );
  await screen.findByText("Several independently supported uses");
  expect(individualDecisionNative.evaluate).toHaveBeenCalledTimes(2);
});

it("ignores a late assessment of an earlier observation instead of showing it as current", async () => {
  let resolveOld!: (value: IndividualDecisionResult) => void;
  vi.mocked(individualDecisionNative.evaluate)
    .mockReturnValueOnce(
      new Promise((resolve) => {
        resolveOld = resolve;
      }),
    )
    .mockResolvedValueOnce(result("2", "Current confirmed evolution"));
  const view = render(
    <IndividualDecisionPanel
      id={id}
      observationRevision="1"
      refreshToken={0}
      onOpenIndividual={vi.fn()}
    />,
  );
  await waitFor(() =>
    expect(individualDecisionNative.evaluate).toHaveBeenCalledOnce(),
  );
  view.rerender(
    <IndividualDecisionPanel
      id={id}
      observationRevision="2"
      refreshToken={0}
      onOpenIndividual={vi.fn()}
    />,
  );
  await screen.findByText("Current confirmed evolution");
  resolveOld(result("1", "Outdated state"));
  await waitFor(() =>
    expect(screen.queryByText("Outdated state")).not.toBeInTheDocument(),
  );
  expect(
    screen.getByText(`Roadmap individual ${id}, revision 2`),
  ).toBeVisible();
});

it("does not display an assessment returned for another individual", async () => {
  const wrong = result();
  wrong.report = {
    ...wrong.report,
    pokemonId: "00000000-0000-4000-8000-000000000002",
  };
  vi.mocked(individualDecisionNative.evaluate).mockResolvedValue(wrong);
  render(
    <IndividualDecisionPanel
      id={id}
      observationRevision="1"
      refreshToken={0}
      onOpenIndividual={vi.fn()}
    />,
  );
  await screen.findByText(/assessment did not match this individual/);
  expect(
    screen.queryByText("Several independently supported uses"),
  ).not.toBeInTheDocument();
});

it("reruns the integrated decision when roadmap allocations or route selection change", async () => {
  vi.mocked(individualDecisionNative.evaluate)
    .mockResolvedValueOnce(result())
    .mockResolvedValueOnce(result("2", "Resource-aware plan updated"));
  render(
    <IndividualDecisionPanel
      id={id}
      observationRevision="1"
      refreshToken={0}
      onOpenIndividual={vi.fn()}
    />,
  );
  await screen.findByText("Several independently supported uses");
  fireEvent.click(
    screen.getByRole("button", {
      name: "Refresh after allocation confirmation",
    }),
  );
  await screen.findByText("Resource-aware plan updated");
  expect(individualDecisionNative.evaluate).toHaveBeenCalledTimes(2);
});
