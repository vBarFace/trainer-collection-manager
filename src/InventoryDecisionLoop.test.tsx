import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { Inventory } from "./Inventory";
import { native, type InventoryRecord, type ScreenshotPreview } from "./native";
import { extractText } from "./ocr";
import {
  individualDecisionNative,
  type IndividualDecisionResult,
} from "./decision-native";
import type { IndividualFacts } from "./IndividualDetails";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("./ocr", () => ({ extractText: vi.fn() }));
vi.mock("./decision-native", () => ({
  individualDecisionNative: { evaluate: vi.fn() },
}));
vi.mock("./native", async (importOriginal) => {
  const actual = await importOriginal<typeof import("./native")>();
  return {
    ...actual,
    native: {
      ...actual.native,
      snapshot: vi.fn(),
      selectScreenshot: vi.fn(),
      confirmImport: vi.fn(),
      confirmPowerUp: vi.fn(),
    },
  };
});
const id = "00000000-0000-4000-8000-000000000001";
const protectedTag = {
  id: "9",
  name: "SAVE/SPECIAL",
  scope: "standard" as const,
};
const individual: InventoryRecord = {
  id,
  displayName: "Fictional Spark",
  cp: "321",
  strategicObservationRevision: "0",
  ivs: { attack: 12, defense: 13, hp: 14 },
  tags: [protectedTag],
  reference: {
    state: "not_set",
    identity: null,
    dataset: {
      datasetId: "fictional",
      version: "one",
      sourceUrl: "https://example.invalid",
      upstreamRevision: "original",
      license: "MIT",
      notice: "Original fictional fixture",
      retrievedAt: "2026-11-02",
      rightsEvidenceUrl: "https://example.invalid/license",
    },
    species: null,
    form: null,
  },
  evaluation: {
    scope: "appraisal",
    state: "pending",
    currentRulesetVersion: "appraisal-1",
    rulesetVersion: null,
    evaluatedAt: null,
    result: null,
  },
};
function decision(summary: string, revision = "1"): IndividualDecisionResult {
  const report: IndividualDecisionResult["report"] = {
    pokemonId: id,
    displayName: individual.displayName,
    rulesetVersion: "fictional-loop-1",
    observationRevision: revision,
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
    steps: [
      {
        id: "next-observation",
        action: "review",
        required: true,
        prerequisites: [],
        rationale: `Next supported step for ${summary}`,
        cost: null,
        executable: true,
      },
    ],
    limitations: [],
    decision: {
      summary,
      suggestions: [
        {
          id: "collection",
          tag: "SAVE / SPECIAL",
          state: "collection_value",
          targetFormId: null,
          targetName: null,
          explanation: "Preserve confirmed personal collection value.",
          evidence: ["Manually applied preservation tag"],
          missing: [],
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
const emptyFacts: IndividualFacts = {
  currentHp: null,
  maximumHp: null,
  weightKg: null,
  heightM: null,
  gender: null,
  catchDate: null,
  fastMove: null,
  chargedMove: null,
  secondChargedMove: null,
  level: null,
  levelEvidence: null,
  shiny: null,
  shadow: null,
  purified: null,
  favorite: null,
  protected: false,
};
const image: ScreenshotPreview = {
  png: [137, 80, 78, 71],
  width: 1080,
  height: 1920,
  textRegions: { displayName: null, cp: null },
  extraction: {
    displayName: {
      value: "Fictional Spark",
      confidence: "tentative",
      method: "tesseract_lstm",
      state: "detected",
    },
    cp: {
      value: "321",
      confidence: "tentative",
      method: "tesseract_lstm",
      state: "detected",
    },
    attack: {
      value: null,
      confidence: "none",
      method: "appraisal_geometry_v1",
      state: "unresolved",
    },
    defense: {
      value: null,
      confidence: "none",
      method: "appraisal_geometry_v1",
      state: "unresolved",
    },
    hp: {
      value: null,
      confidence: "none",
      method: "appraisal_geometry_v1",
      state: "unresolved",
    },
    issues: [],
  },
};

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(native.snapshot).mockResolvedValue({
    records: [individual],
    tags: [protectedTag],
  });
  vi.mocked(native.confirmPowerUp).mockResolvedValue({ ...individual });
  vi.mocked(individualDecisionNative.evaluate).mockResolvedValue(
    decision("Initial individual analysis"),
  );
  vi.mocked(extractText).mockImplementation(
    async (selected) => selected.extraction,
  );
  Object.defineProperty(URL, "createObjectURL", {
    configurable: true,
    value: vi.fn().mockReturnValue("blob:fictional-import"),
  });
  Object.defineProperty(URL, "revokeObjectURL", {
    configurable: true,
    value: vi.fn(),
  });
});
afterEach(cleanup);

it("automatically selects and assesses a confirmed image import with unknown optional facts", async () => {
  vi.mocked(native.snapshot).mockResolvedValue({ records: [], tags: [] });
  vi.mocked(native.selectScreenshot).mockResolvedValue(image);
  vi.mocked(native.confirmImport).mockResolvedValue({
    ...individual,
    tags: [],
    ivs: null,
  });
  render(<Inventory />);
  await waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Select screenshot" }),
    ).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Select screenshot" }));
  await screen.findByDisplayValue("Fictional Spark");
  await waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Confirm and save" }),
    ).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Confirm and save" }));
  await screen.findByText("Initial individual analysis");
  expect(native.confirmImport).toHaveBeenCalledExactlyOnceWith({
    displayName: "Fictional Spark",
    cp: "321",
    ivs: null,
    tagIds: [],
  });
  expect(individualDecisionNative.evaluate).toHaveBeenCalledExactlyOnceWith(id);
  expect(screen.getByText("Tags: None")).toBeVisible();
  expect(screen.queryByLabelText("Collection goal")).not.toBeInTheDocument();
  expect(
    screen.getByText("Next supported step for Initial individual analysis"),
  ).toBeVisible();
});

it("refreshes the same individual's roadmap after a history-only Power Up without changing applied tags", async () => {
  vi.mocked(individualDecisionNative.evaluate)
    .mockResolvedValueOnce(decision("Initial individual analysis"))
    .mockResolvedValueOnce(
      decision("Remaining path after confirmed Power Up", "2"),
    );
  render(<Inventory initialSelectedId={id} />);
  await screen.findByText("Initial individual analysis");
  fireEvent.click(screen.getByRole("button", { name: "Power Up" }));
  fireEvent.click(screen.getByRole("button", { name: "Review Power Up" }));
  fireEvent.click(
    screen.getByLabelText("I confirm I performed this Power Up in Pokémon GO"),
  );
  fireEvent.click(screen.getByRole("button", { name: "Confirm Power Up" }));
  await screen.findByText("Power Up confirmed and saved locally.");
  await screen.findByText("Remaining path after confirmed Power Up");
  expect(native.confirmPowerUp).toHaveBeenCalledExactlyOnceWith(
    id,
    expect.objectContaining({
      expectedStrategicObservationRevision: "0",
      cp: null,
      performedInGameConfirmed: true,
    }),
  );
  expect(vi.mocked(individualDecisionNative.evaluate).mock.calls).toEqual([
    [id],
    [id],
  ]);
  expect(
    screen.getByText(
      "Next supported step for Remaining path after confirmed Power Up",
    ),
  ).toBeVisible();
  expect(screen.getByText("Tags: SAVE/SPECIAL")).toBeVisible();
  expect(screen.getByText("CP: 321")).toBeVisible();
});

it("does not turn a saved game action into a failure when automatic analysis is unavailable", async () => {
  vi.mocked(individualDecisionNative.evaluate)
    .mockResolvedValueOnce(decision("Initial individual analysis"))
    .mockRejectedValueOnce(new Error("Temporary assessment failure"))
    .mockResolvedValueOnce(decision("Assessment recovered", "2"));
  render(<Inventory initialSelectedId={id} />);
  await screen.findByText("Initial individual analysis");
  fireEvent.click(screen.getByRole("button", { name: "Power Up" }));
  fireEvent.click(screen.getByRole("button", { name: "Review Power Up" }));
  fireEvent.click(
    screen.getByLabelText("I confirm I performed this Power Up in Pokémon GO"),
  );
  fireEvent.click(screen.getByRole("button", { name: "Confirm Power Up" }));
  await screen.findByText("Power Up confirmed and saved locally.");
  await screen.findByText(/Recommendation refresh unavailable/);
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  fireEvent.click(
    screen.getByRole("button", { name: "Retry individual assessment" }),
  );
  await screen.findByText("Assessment recovered");
  expect(native.confirmPowerUp).toHaveBeenCalledTimes(1);
  expect(individualDecisionNative.evaluate).toHaveBeenCalledTimes(3);
});

it("reevaluates confirmed optional details while keeping blank facts unknown", async () => {
  vi.mocked(native.snapshot)
    .mockResolvedValueOnce({ records: [individual], tags: [protectedTag] })
    .mockResolvedValue({
      records: [{ ...individual, strategicObservationRevision: "1" }],
      tags: [protectedTag],
    });
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "individual_details")
      return {
        values: emptyFacts,
        observationRevision: "0",
        needsReconfirmation: false,
        evidenceToken: "evidence-one",
      };
    if (command === "save_individual_details") return null;
    throw new Error(`Unexpected command ${command}`);
  });
  vi.mocked(individualDecisionNative.evaluate)
    .mockResolvedValueOnce(decision("Initial individual analysis"))
    .mockResolvedValue(decision("Confirmed collection property reviewed", "2"));
  render(<Inventory initialSelectedId={id} />);
  await screen.findByText("Initial individual analysis");
  fireEvent.click(
    screen.getByRole("button", { name: "Review optional observations" }),
  );
  await screen.findByLabelText("Shiny");
  fireEvent.change(screen.getByLabelText("Shiny"), {
    target: { value: "true" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Confirm optional observations" }),
  );
  await screen.findByText(
    "Optional observations confirmed locally. Blank fields remain unknown.",
  );
  await screen.findByText("Confirmed collection property reviewed");
  expect(invoke).toHaveBeenCalledWith(
    "save_individual_details",
    expect.objectContaining({
      id,
      values: expect.objectContaining({
        shiny: true,
        weightKg: null,
        heightM: null,
        fastMove: null,
        level: null,
      }),
    }),
  );
  expect(screen.getByText("Tags: SAVE/SPECIAL")).toBeVisible();
});
