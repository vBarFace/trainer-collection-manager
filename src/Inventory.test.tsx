import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Inventory } from "./Inventory";
import { native, type InventoryRecord, type ScreenshotPreview } from "./native";
import { emptyReview, validateReview } from "./review";
import { extractText } from "./ocr";

vi.mock("./ocr", () => ({ extractText: vi.fn() }));

vi.mock("./native", async (importOriginal) => {
  const actual = await importOriginal<typeof import("./native")>();
  return {
    ...actual,
    native: {
      snapshot: vi.fn(),
      selectScreenshot: vi.fn(),
      confirmImport: vi.fn(),
      evaluate: vi.fn(),
      correctIvs: vi.fn(),
    },
  };
});

const special = { id: "9", name: "SAVE/SPECIAL", scope: "standard" as const };
const custom = { id: "10", name: "Favorites", scope: "local" as const };
const image: ScreenshotPreview = {
  png: [137, 80, 78, 71],
  width: 1080,
  height: 1920,
  textRegions: { displayName: null, cp: null },
  extraction: {
    displayName: {
      value: null,
      confidence: "none",
      method: "unavailable",
      state: "unavailable",
    },
    cp: {
      value: null,
      confidence: "none",
      method: "unavailable",
      state: "unavailable",
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
    issues: [
      {
        code: "bars_unresolved",
        message: "Appraisal bars unresolved. Manual entry remains available.",
      },
    ],
  },
};
const saved: InventoryRecord = {
  id: "00000000-0000-4000-8000-000000000000",
  displayName: "Buddy",
  cp: "9223372036854775807",
  ivs: { attack: 0, defense: 14, hp: 15 },
  tags: [special, custom],
  evaluation: {
    scope: "appraisal",
    state: "pending",
    currentRulesetVersion: "appraisal-1",
    rulesetVersion: null,
    evaluatedAt: null,
    result: null,
  },
};
const imported: InventoryRecord = { ...saved, tags: [] };
let createObjectURL: ReturnType<typeof vi.fn>;
let revokeObjectURL: ReturnType<typeof vi.fn>;

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(extractText).mockImplementation(
    async (selected) => selected.extraction,
  );
  vi.mocked(native.snapshot).mockResolvedValue({
    records: [],
    tags: [special, custom],
  });
  vi.mocked(native.selectScreenshot).mockResolvedValue(image);
  vi.mocked(native.confirmImport).mockResolvedValue(imported);
  createObjectURL = vi.fn().mockReturnValue("blob:local-review");
  revokeObjectURL = vi.fn();
  Object.defineProperty(URL, "createObjectURL", {
    configurable: true,
    value: createObjectURL,
  });
  Object.defineProperty(URL, "revokeObjectURL", {
    configurable: true,
    value: revokeObjectURL,
  });
});
afterEach(cleanup);

async function selectImage() {
  await waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Select screenshot" }),
    ).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Select screenshot" }));
  await screen.findByRole("heading", { name: "Review and correct details" });
}
function fillName() {
  fireEvent.change(screen.getByLabelText("Display name (required)"), {
    target: { value: "Buddy" },
  });
}
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

describe("manual review validation", () => {
  it("keeps unknown IVs absent and accepts complete groups including zero", () => {
    expect(validateReview({ ...emptyReview, displayName: "Buddy" })).toEqual({
      displayName: "Buddy",
      cp: null,
      ivs: null,
      tagIds: [],
    });
    expect(
      validateReview({
        ...emptyReview,
        displayName: "Buddy",
        attack: "0",
        defense: "14",
        hp: "15",
      }),
    ).toMatchObject({ ivs: { attack: 0, defense: 14, hp: 15 } });
  });
  it("rejects every partial IV group and invalid IV values", () => {
    for (let mask = 1; mask < 7; mask++)
      expect(
        typeof validateReview({
          ...emptyReview,
          displayName: "Buddy",
          attack: mask & 1 ? "1" : "",
          defense: mask & 2 ? "2" : "",
          hp: mask & 4 ? "3" : "",
        }),
      ).toBe("string");
    for (const value of ["16", "-1", "1.5", "1e1", " "])
      expect(
        typeof validateReview({
          ...emptyReview,
          displayName: "Buddy",
          attack: value,
          defense: "1",
          hp: "2",
        }),
      ).toBe("string");
  });
  it("preserves exact CP above JS precision and enforces the SQLite boundary", () => {
    for (const cp of ["9007199254740993", "9223372036854775807"])
      expect(
        validateReview({ ...emptyReview, displayName: "Buddy", cp }),
      ).toMatchObject({ cp });
    for (const cp of ["0", "-1", "1.5", "1e2", "9223372036854775808", " 1"])
      expect(
        typeof validateReview({ ...emptyReview, displayName: "Buddy", cp }),
      ).toBe("string");
  });
  it("validates Unicode name length, whitespace and controls", () => {
    expect(
      typeof validateReview({ ...emptyReview, displayName: "😀".repeat(100) }),
    ).toBe("object");
    for (const displayName of ["", " Buddy", "Buddy ", "a\nb", "x".repeat(101)])
      expect(typeof validateReview({ ...emptyReview, displayName })).toBe(
        "string",
      );
  });
});

describe("Inventory manual import", () => {
  it("keeps retrieved records pending across remount and preserves existing tags", async () => {
    vi.mocked(native.snapshot).mockResolvedValue({
      records: [saved],
      tags: [special, custom],
    });
    render(<Inventory />);
    expect(
      await screen.findByRole("heading", { name: "Buddy" }),
    ).toBeInTheDocument();
    expect(screen.getByText("CP: 9223372036854775807")).toBeInTheDocument();
    expect(
      screen.getByText("Appraisal IVs: 0 / 14 / 15 (attack / defense / HP)"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Tags: SAVE/SPECIAL, Favorites"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Local collection" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Evaluation: Pending")).toBeInTheDocument();
    cleanup();
    render(<Inventory />);
    await screen.findByRole("heading", { name: "Buddy" });
    expect(screen.getByText("Evaluation: Pending")).toBeInTheDocument();
    expect(
      screen.getByText("Tags: SAVE/SPECIAL, Favorites"),
    ).toBeInTheDocument();
    expect(native.confirmImport).not.toHaveBeenCalled();
  });
  it("does not persist during review or cancellation and releases preview", async () => {
    render(<Inventory />);
    await selectImage();
    fillName();
    expect(native.confirmImport).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Cancel review" }));
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
    expect(revokeObjectURL).toHaveBeenCalledWith("blob:local-review");
    expect(native.confirmImport).not.toHaveBeenCalled();
    await selectImage();
    expect(screen.getByLabelText("Display name (required)")).toHaveValue("");
  });
  it("saves confirmed data without tag assignments and displays the record awaiting evaluation", async () => {
    render(<Inventory />);
    await selectImage();
    fillName();
    fireEvent.change(screen.getByLabelText("CP (optional)"), {
      target: { value: "9223372036854775807" },
    });
    fireEvent.change(screen.getByLabelText("Attack IV"), {
      target: { value: "0" },
    });
    fireEvent.change(screen.getByLabelText("Defense IV"), {
      target: { value: "14" },
    });
    fireEvent.change(screen.getByLabelText("HP IV"), {
      target: { value: "15" },
    });
    expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Confirm and save" }));
    await screen.findByText(
      "Saved Buddy to your local inventory. Awaiting evaluation.",
    );
    expect(native.confirmImport).toHaveBeenCalledExactlyOnceWith({
      displayName: "Buddy",
      cp: "9223372036854775807",
      ivs: { attack: 0, defense: 14, hp: 15 },
      tagIds: [],
    });
    expect(screen.getByRole("heading", { name: "Buddy" })).toBeInTheDocument();
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
    expect(revokeObjectURL).toHaveBeenCalledTimes(1);
    expect(native.snapshot).toHaveBeenCalledTimes(1);
    expect(screen.getByText("Evaluation: Pending")).toBeInTheDocument();
    expect(screen.getByText("Tags: None")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Select screenshot" }),
    ).toHaveFocus();
  });
  it("blocks incomplete groups before calling native save", async () => {
    render(<Inventory />);
    await selectImage();
    fillName();
    fireEvent.change(screen.getByLabelText("Attack IV"), {
      target: { value: "15" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Confirm and save" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Leave all three appraisal IVs empty",
    );
    expect(native.confirmImport).not.toHaveBeenCalled();
  });
  it("prevents repeated submit while saving and retains values after recoverable failure", async () => {
    const pending = deferred<InventoryRecord>();
    vi.mocked(native.confirmImport).mockReturnValueOnce(pending.promise);
    render(<Inventory />);
    await selectImage();
    fillName();
    fireEvent.change(screen.getByLabelText("CP (optional)"), {
      target: { value: "9007199254740993" },
    });
    fireEvent.change(screen.getByLabelText("Attack IV"), {
      target: { value: "0" },
    });
    fireEvent.change(screen.getByLabelText("Defense IV"), {
      target: { value: "14" },
    });
    fireEvent.change(screen.getByLabelText("HP IV"), {
      target: { value: "15" },
    });
    const submit = screen.getByRole("button", { name: "Confirm and save" });
    const form = submit.closest("form")!;
    fireEvent.submit(form);
    fireEvent.submit(form);
    expect(native.confirmImport).toHaveBeenCalledTimes(1);
    expect(submit).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Cancel review" }),
    ).toBeDisabled();
    await act(async () =>
      pending.reject({
        code: "storage",
        message: "Storage unavailable; try again.",
      }),
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Storage unavailable; try again.",
    );
    expect(screen.getByLabelText("Display name (required)")).toHaveValue(
      "Buddy",
    );
    expect(screen.getByLabelText("CP (optional)")).toHaveValue(
      "9007199254740993",
    );
    expect(screen.getByLabelText("Attack IV")).toHaveValue("0");
    expect(screen.getByLabelText("Defense IV")).toHaveValue("14");
    expect(screen.getByLabelText("HP IV")).toHaveValue("15");
    expect(screen.getByRole("img")).toBeInTheDocument();
    expect(revokeObjectURL).not.toHaveBeenCalled();
    fireEvent.click(submit);
    await screen.findByText(
      "Saved Buddy to your local inventory. Awaiting evaluation.",
    );
    expect(native.confirmImport).toHaveBeenCalledTimes(2);
  });
  it("releases replaced preview even when native picker is cancelled", async () => {
    render(<Inventory />);
    await selectImage();
    vi.mocked(native.selectScreenshot).mockResolvedValueOnce(null);
    fireEvent.click(screen.getByRole("button", { name: "Replace screenshot" }));
    await screen.findByText(
      "Screenshot selection cancelled. Nothing was saved.",
    );
    expect(revokeObjectURL).toHaveBeenCalledTimes(1);
    expect(
      screen.getByRole("button", { name: "Select screenshot" }),
    ).toHaveFocus();
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
    expect(native.confirmImport).not.toHaveBeenCalled();
  });
  it("releases preview on unmount and ignores a late picker completion", async () => {
    const view = render(<Inventory />);
    await selectImage();
    view.unmount();
    expect(revokeObjectURL).toHaveBeenCalledTimes(1);
    const pending = deferred<ScreenshotPreview | null>();
    vi.mocked(native.selectScreenshot).mockReturnValueOnce(pending.promise);
    const next = render(<Inventory />);
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Select screenshot" }),
      ).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Select screenshot" }));
    next.unmount();
    await act(async () => pending.resolve(image));
    expect(createObjectURL).toHaveBeenCalledTimes(1);
  });
  it("shows safe errors for raw failures and allows inventory retry", async () => {
    vi.mocked(native.snapshot).mockRejectedValueOnce(
      "private local path or SQL",
    );
    render(<Inventory />);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Unable to load the local inventory.",
    );
    expect(
      screen.queryByText("private local path or SQL"),
    ).not.toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "Retry loading inventory" }),
    );
    await selectImage();
  });
});

describe("persisted appraisal evaluation", () => {
  const evaluated: InventoryRecord = {
    ...saved,
    evaluation: {
      scope: "appraisal",
      state: "evaluated",
      currentRulesetVersion: "appraisal-1",
      rulesetVersion: "appraisal-1",
      evaluatedAt: "2026-10-26 12:00:00",
      result: {
        ivs: { attack: 15, defense: 15, hp: 15 },
        total: 45,
        appraisalPercentage: 100,
        perfect: true,
        findings: [
          {
            code: "perfect_appraisal",
            explanation:
              "Perfect appraisal does not imply universal competitive superiority.",
            observedFacts: [
              { field: "attack", value: 15 },
              { field: "defense", value: 15 },
              { field: "hp", value: 15 },
            ],
          },
        ],
        recommendations: [],
        missingData: [],
        limitations: [
          "Appraisal percentage is descriptive only, not a PvP ranking or transfer criterion.",
        ],
      },
    },
    ivs: { attack: 15, defense: 15, hp: 15 },
  };
  async function selectRecord() {
    fireEvent.click(
      await screen.findByRole("button", { name: "Review appraisal for Buddy" }),
    );
    expect(
      screen.getByRole("heading", { name: "Appraisal for Buddy" }),
    ).toHaveFocus();
  }
  it("renders all four native lifecycle states and an empty collection", async () => {
    render(<Inventory />);
    expect(await screen.findByText(/No Pokémon saved yet/)).toBeInTheDocument();
    cleanup();
    vi.mocked(native.snapshot).mockResolvedValue({
      records: ["pending", "evaluated", "needs_data", "stale"].map(
        (state, index) => ({
          ...saved,
          id: String(index),
          displayName: state,
          evaluation: {
            ...saved.evaluation,
            state: state as InventoryRecord["evaluation"]["state"],
          },
        }),
      ),
      tags: [],
    });
    render(<Inventory />);
    for (const label of ["Pending", "Evaluated", "Needs data", "Stale"])
      expect(
        await screen.findByText(`Evaluation: ${label}`),
      ).toBeInTheDocument();
    expect(native.evaluate).not.toHaveBeenCalled();
  });
  it("requests only a selected ID, prevents duplicate requests and displays native results separately from tags", async () => {
    vi.mocked(native.snapshot).mockResolvedValue({
      records: [saved],
      tags: [special, custom],
    });
    const pending = deferred<InventoryRecord>();
    vi.mocked(native.evaluate).mockReturnValueOnce(pending.promise);
    render(<Inventory />);
    await selectRecord();
    const button = screen.getByRole("button", { name: "Evaluate appraisal" });
    fireEvent.click(button);
    fireEvent.click(button);
    expect(button).toBeDisabled();
    expect(
      screen.getByText("Evaluating confirmed appraisal IVs locally…"),
    ).toBeInTheDocument();
    expect(native.evaluate).toHaveBeenCalledExactlyOnceWith(saved.id);
    await act(async () => pending.resolve(evaluated));
    expect(screen.getByText("Evaluation: Evaluated")).toBeInTheDocument();
    expect(
      screen.getByText("Total appraisal IVs: 45 / 45"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Appraisal percentage: 100.00%"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Perfect 15/15/15 appraisal: Yes"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/does not imply universal competitive/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/No action recommendations or suggested tags/),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Assigned tags" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Tags: SAVE/SPECIAL, Favorites"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/not a PvP ranking or transfer criterion/),
    ).toBeInTheDocument();
    expect(native.confirmImport).not.toHaveBeenCalled();
    cleanup();
    vi.mocked(native.snapshot).mockResolvedValue({
      records: [evaluated],
      tags: [special, custom],
    });
    render(<Inventory />);
    await selectRecord();
    expect(
      screen.getByText("Appraisal percentage: 100.00%"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/2026-10-26 12:00:00 UTC · appraisal-1/),
    ).toBeInTheDocument();
    expect(native.evaluate).toHaveBeenCalledTimes(1);
  });
  it("shows missing data explicitly and permits re-evaluation", async () => {
    const missing: InventoryRecord = {
      ...saved,
      ivs: null,
      evaluation: {
        ...evaluated.evaluation,
        state: "needs_data",
        result: {
          ...evaluated.evaluation.result!,
          ivs: null,
          total: null,
          appraisalPercentage: null,
          perfect: null,
          findings: [],
          missingData: ["attack", "defense", "hp"],
        },
      },
    };
    vi.mocked(native.snapshot).mockResolvedValue({
      records: [missing],
      tags: [],
    });
    vi.mocked(native.evaluate).mockResolvedValue(missing);
    render(<Inventory />);
    await selectRecord();
    expect(
      screen.getByText(
        /Required confirmed IVs are unknown: attack, defense, HP/,
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/Appraisal percentage: /),
    ).not.toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "Re-evaluate appraisal" }),
    );
    await waitFor(() =>
      expect(native.evaluate).toHaveBeenCalledExactlyOnceWith(saved.id),
    );
  });
  it("withholds stale results and recovers from safe failure with retry", async () => {
    const stale: InventoryRecord = {
      ...saved,
      evaluation: {
        ...evaluated.evaluation,
        state: "stale",
        rulesetVersion: "old-rules",
        result: null,
      },
    };
    vi.mocked(native.snapshot).mockResolvedValue({
      records: [stale],
      tags: [],
    });
    vi.mocked(native.evaluate)
      .mockRejectedValueOnce("private path or SQL")
      .mockResolvedValueOnce(evaluated);
    render(<Inventory />);
    await selectRecord();
    expect(screen.getByText(/previous result is withheld/)).toBeInTheDocument();
    expect(
      screen.queryByText(/Appraisal percentage: /),
    ).not.toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "Re-evaluate appraisal" }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Unable to evaluate appraisal.",
    );
    expect(screen.queryByText("private path or SQL")).not.toBeInTheDocument();
    expect(screen.getByText("Evaluation: Stale")).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "Retry appraisal evaluation" }),
    );
    await screen.findByText("Evaluation: Evaluated");
    expect(native.evaluate).toHaveBeenCalledTimes(2);
  });
  it("allows reloading after deletion and ignores a late result on unmount", async () => {
    vi.mocked(native.snapshot)
      .mockResolvedValueOnce({ records: [saved], tags: [] })
      .mockResolvedValue({ records: [], tags: [] });
    vi.mocked(native.evaluate).mockRejectedValueOnce({
      code: "missing_pokemon",
      message: "This Pokémon no longer exists. Reload the inventory.",
    });
    const view = render(<Inventory />);
    await selectRecord();
    fireEvent.click(screen.getByRole("button", { name: "Evaluate appraisal" }));
    await screen.findByRole("alert");
    fireEvent.click(screen.getByRole("button", { name: "Reload inventory" }));
    await screen.findByText(/No Pokémon saved yet/);
    expect(
      screen.queryByRole("heading", { name: "Appraisal for Buddy" }),
    ).not.toBeInTheDocument();
    view.unmount();
    vi.mocked(native.snapshot).mockResolvedValue({
      records: [saved],
      tags: [],
    });
    const pending = deferred<InventoryRecord>();
    vi.mocked(native.evaluate).mockReturnValueOnce(pending.promise);
    const next = render(<Inventory />);
    await selectRecord();
    fireEvent.click(screen.getByRole("button", { name: "Evaluate appraisal" }));
    next.unmount();
    await act(async () => pending.resolve(evaluated));
    expect(native.confirmImport).not.toHaveBeenCalled();
  });
});

describe("confirmed saved-IV corrections", () => {
  const appraised: InventoryRecord = {
    ...saved,
    evaluation: {
      ...saved.evaluation,
      state: "evaluated",
      rulesetVersion: "appraisal-1",
      evaluatedAt: "2026-10-26 12:00:00",
      result: {
        ivs: saved.ivs,
        total: 29,
        appraisalPercentage: (29 * 100) / 45,
        perfect: false,
        findings: [],
        recommendations: [],
        missingData: [],
        limitations: ["Appraisal is descriptive only."],
      },
    },
  };
  const corrected: InventoryRecord = {
    ...appraised,
    ivs: { attack: 15, defense: 15, hp: 15 },
    evaluation: { ...appraised.evaluation, state: "stale", result: null },
  };
  async function openCorrection() {
    fireEvent.click(
      await screen.findByRole("button", { name: "Review appraisal for Buddy" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Correct saved IVs" }));
    expect(screen.getByLabelText("Corrected attack IV")).toHaveFocus();
  }
  function changeIvs(attack: string, defense: string, hp: string) {
    for (const [label, value] of [
      ["Corrected attack IV", attack],
      ["Corrected defense IV", defense],
      ["Corrected HP IV", hp],
    ])
      fireEvent.change(screen.getByLabelText(label), { target: { value } });
  }
  beforeEach(() => {
    vi.mocked(native.snapshot).mockResolvedValue({
      records: [appraised],
      tags: [special, custom],
    });
    vi.mocked(native.correctIvs).mockResolvedValue(corrected);
  });
  it("prefills saved IVs, keeps edits as drafts and cancels without writes", async () => {
    render(<Inventory />);
    await openCorrection();
    expect(screen.getByLabelText("Corrected attack IV")).toHaveValue("0");
    expect(screen.getByLabelText("Corrected defense IV")).toHaveValue("14");
    expect(screen.getByLabelText("Corrected HP IV")).toHaveValue("15");
    expect(
      screen.getByRole("button", { name: "Re-evaluate appraisal" }),
    ).toBeDisabled();
    changeIvs("15", "15", "15");
    expect(screen.getByText("Evaluation: Evaluated")).toBeInTheDocument();
    expect(native.correctIvs).not.toHaveBeenCalled();
    fireEvent.click(
      screen.getByRole("button", { name: "Cancel IV correction" }),
    );
    expect(
      screen.getByRole("button", { name: "Correct saved IVs" }),
    ).toHaveFocus();
    expect(
      screen.queryByLabelText("Corrected attack IV"),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Correct saved IVs" }));
    expect(screen.getByLabelText("Corrected attack IV")).toHaveValue("0");
    expect(native.confirmImport).not.toHaveBeenCalled();
    expect(native.correctIvs).not.toHaveBeenCalled();
    expect(native.evaluate).not.toHaveBeenCalled();
  });
  it("validates groups before saving and requires confirmation before clearing to unknown", async () => {
    const cleared: InventoryRecord = { ...corrected, ivs: null };
    vi.mocked(native.correctIvs).mockResolvedValue(cleared);
    render(<Inventory />);
    await openCorrection();
    for (const attack of ["16", "-1", "1.5", "1e1", " "]) {
      changeIvs(attack, "14", "15");
      fireEvent.click(
        screen.getByRole("button", { name: "Confirm IV correction" }),
      );
      expect(await screen.findByRole("alert")).toHaveTextContent(
        "Leave all three appraisal IVs empty",
      );
    }
    changeIvs("0", "", "15");
    fireEvent.click(
      screen.getByRole("button", { name: "Confirm IV correction" }),
    );
    expect(native.correctIvs).not.toHaveBeenCalled();
    fireEvent.click(
      screen.getByRole("button", { name: "Mark all IVs unknown" }),
    );
    for (const label of [
      "Corrected attack IV",
      "Corrected defense IV",
      "Corrected HP IV",
    ])
      expect(screen.getByLabelText(label)).toHaveValue("");
    expect(native.correctIvs).not.toHaveBeenCalled();
    fireEvent.click(
      screen.getByRole("button", { name: "Confirm IV correction" }),
    );
    await screen.findByText(/IV correction saved/);
    expect(native.correctIvs).toHaveBeenCalledExactlyOnceWith(
      saved.id,
      saved.ivs,
      null,
    );
    expect(screen.getByText("Appraisal IVs: Unknown")).toBeInTheDocument();
    expect(screen.getByText("Evaluation: Stale")).toBeInTheDocument();
    expect(native.evaluate).not.toHaveBeenCalled();
  });
  it("saves only confirmed IVs once, shows stale state and permits a separate re-evaluation", async () => {
    const pending = deferred<InventoryRecord>();
    vi.mocked(native.correctIvs).mockReturnValueOnce(pending.promise);
    vi.mocked(native.evaluate).mockResolvedValue({
      ...corrected,
      evaluation: {
        ...appraised.evaluation,
        result: {
          ...appraised.evaluation.result!,
          ivs: corrected.ivs,
          total: 45,
          appraisalPercentage: 100,
          perfect: true,
        },
      },
    });
    render(<Inventory />);
    await openCorrection();
    changeIvs("15", "15", "15");
    const form = screen.getByRole("form", {
      name: "Correct saved appraisal IVs",
    });
    fireEvent.submit(form);
    fireEvent.submit(form);
    expect(native.correctIvs).toHaveBeenCalledExactlyOnceWith(
      saved.id,
      saved.ivs,
      corrected.ivs,
    );
    expect(screen.getByLabelText("Corrected attack IV")).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Cancel IV correction" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Review appraisal for Buddy" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Re-evaluate appraisal" }),
    ).toBeDisabled();
    expect(
      screen.getByText("Saving confirmed IV corrections locally…"),
    ).toBeInTheDocument();
    await act(async () => pending.resolve(corrected));
    expect(screen.getByText("Evaluation: Stale")).toBeInTheDocument();
    expect(
      screen.getByText("Appraisal IVs: 15 / 15 / 15 (attack / defense / HP)"),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("Total appraisal IVs: 29 / 45"),
    ).not.toBeInTheDocument();
    expect(screen.getByText("CP: 9223372036854775807")).toBeInTheDocument();
    expect(
      screen.getByText("Tags: SAVE/SPECIAL, Favorites"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Correct saved IVs" }),
    ).toHaveFocus();
    expect(native.evaluate).not.toHaveBeenCalled();
    expect(native.confirmImport).not.toHaveBeenCalled();
    fireEvent.click(
      screen.getByRole("button", { name: "Re-evaluate appraisal" }),
    );
    await screen.findByText("Appraisal percentage: 100.00%");
    expect(native.evaluate).toHaveBeenCalledExactlyOnceWith(saved.id);
    cleanup();
    vi.mocked(native.snapshot).mockResolvedValue({
      records: [corrected],
      tags: [special, custom],
    });
    render(<Inventory />);
    await openCorrection();
    expect(screen.getByLabelText("Corrected attack IV")).toHaveValue("15");
    expect(screen.getByText("Evaluation: Stale")).toBeInTheDocument();
  });
  it("retains corrected values on a safe failure and saves them on retry", async () => {
    vi.mocked(native.correctIvs).mockRejectedValueOnce("private path or SQL");
    render(<Inventory />);
    await openCorrection();
    changeIvs("15", "15", "15");
    fireEvent.click(
      screen.getByRole("button", { name: "Confirm IV correction" }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Your draft is retained; try again.",
    );
    expect(screen.queryByText("private path or SQL")).not.toBeInTheDocument();
    expect(screen.getByLabelText("Corrected attack IV")).toHaveValue("15");
    expect(screen.getByText("Evaluation: Evaluated")).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "Confirm IV correction" }),
    );
    await screen.findByText(/IV correction saved/);
    expect(native.correctIvs).toHaveBeenCalledTimes(2);
    expect(native.correctIvs).toHaveBeenLastCalledWith(
      saved.id,
      saved.ivs,
      corrected.ivs,
    );
  });
  it.each(["conflict", "missing_pokemon"])(
    "requires reload after %s without overwriting saved data",
    async (code) => {
      vi.mocked(native.correctIvs).mockRejectedValueOnce({
        code,
        message: "Reload the inventory before saving again.",
      });
      vi.mocked(native.snapshot)
        .mockResolvedValueOnce({ records: [appraised], tags: [] })
        .mockResolvedValue({
          records: code === "conflict" ? [corrected] : [],
          tags: [],
        });
      render(<Inventory />);
      await openCorrection();
      changeIvs("0", "0", "0");
      fireEvent.click(
        screen.getByRole("button", { name: "Confirm IV correction" }),
      );
      await screen.findByRole("alert");
      expect(screen.getByLabelText("Corrected attack IV")).toHaveValue("0");
      expect(
        screen.getByRole("button", { name: "Confirm IV correction" }),
      ).toBeDisabled();
      fireEvent.click(
        screen.getByRole("button", {
          name: "Discard corrections and reload inventory",
        }),
      );
      expect(native.correctIvs).toHaveBeenCalledTimes(1);
      if (code === "conflict") {
        await screen.findByText("Evaluation: Stale");
        fireEvent.click(
          screen.getByRole("button", { name: "Correct saved IVs" }),
        );
        expect(screen.getByLabelText("Corrected attack IV")).toHaveValue("15");
      } else
        expect(
          await screen.findByText(/No Pokémon saved yet/),
        ).toBeInTheDocument();
    },
  );
  it("can fill an unknown group and discards unsaved drafts when selecting another record", async () => {
    const unknown: InventoryRecord = { ...saved, ivs: null };
    const second: InventoryRecord = {
      ...saved,
      id: "00000000-0000-4000-8000-000000000001",
      displayName: "Second",
    };
    vi.mocked(native.snapshot).mockResolvedValue({
      records: [unknown, second],
      tags: [],
    });
    render(<Inventory />);
    await openCorrection();
    expect(screen.getByLabelText("Corrected attack IV")).toHaveValue("");
    changeIvs("0", "15", "15");
    fireEvent.click(
      screen.getByRole("button", { name: "Review appraisal for Second" }),
    );
    expect(
      screen.queryByLabelText("Corrected attack IV"),
    ).not.toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "Review appraisal for Buddy" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Correct saved IVs" }));
    expect(screen.getByLabelText("Corrected attack IV")).toHaveValue("");
    changeIvs("0", "15", "15");
    fireEvent.click(
      screen.getByRole("button", { name: "Confirm IV correction" }),
    );
    await screen.findByText(/IV correction saved/);
    expect(native.correctIvs).toHaveBeenCalledExactlyOnceWith(saved.id, null, {
      attack: 0,
      defense: 15,
      hp: 15,
    });
  });
  it("ignores late correction completion after unmount", async () => {
    const pending = deferred<InventoryRecord>();
    vi.mocked(native.correctIvs).mockReturnValueOnce(pending.promise);
    const view = render(<Inventory />);
    await openCorrection();
    changeIvs("15", "15", "15");
    fireEvent.click(
      screen.getByRole("button", { name: "Confirm IV correction" }),
    );
    view.unmount();
    await act(async () => pending.resolve(corrected));
    expect(native.evaluate).not.toHaveBeenCalled();
    expect(native.confirmImport).not.toHaveBeenCalled();
  });
});

describe("local extraction review", () => {
  it("cancels OCR during initialization without saving or retaining the preview", async () => {
    vi.mocked(extractText).mockImplementationOnce(
      (_image, signal) =>
        new Promise((_resolve, reject) => {
          signal.addEventListener(
            "abort",
            () => reject(new DOMException("Cancelled", "AbortError")),
            { once: true },
          );
        }),
    );
    render(<Inventory />);
    await selectImage();
    fireEvent.click(screen.getByRole("button", { name: "Cancel extraction" }));
    await screen.findByText("Extraction cancelled. Nothing was saved.");
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
    expect(native.confirmImport).not.toHaveBeenCalled();
    expect(revokeObjectURL).toHaveBeenCalledTimes(1);
  });
  it("aborts pending OCR on unmount and ignores a late result", async () => {
    const pending = deferred<ScreenshotPreview["extraction"]>();
    let signal: AbortSignal | undefined;
    vi.mocked(extractText).mockImplementationOnce((_image, current) => {
      signal = current;
      return pending.promise;
    });
    const view = render(<Inventory />);
    await selectImage();
    view.unmount();
    expect(signal?.aborted).toBe(true);
    await act(async () => pending.resolve(image.extraction));
    expect(native.confirmImport).not.toHaveBeenCalled();
    expect(revokeObjectURL).toHaveBeenCalledTimes(1);
  });
  it("prefills OCR name and CP, permits correction and saves only after confirmation", async () => {
    vi.mocked(extractText).mockResolvedValueOnce({
      ...image.extraction,
      displayName: {
        value: "Abra",
        method: "tesseract_lstm",
        state: "detected",
        confidence: "tentative",
        engineConfidence: 94,
      },
      cp: {
        value: "468",
        method: "tesseract_lstm",
        state: "detected",
        confidence: "tentative",
        engineConfidence: 95,
      },
    });
    render(<Inventory />);
    await selectImage();
    await waitFor(() =>
      expect(screen.getByLabelText("Display name (required)")).toHaveValue(
        "Abra",
      ),
    );
    expect(screen.getByLabelText("CP (optional)")).toHaveValue("468");
    expect(native.confirmImport).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText("CP (optional)"), {
      target: { value: "469" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Confirm and save" }));
    await screen.findByText(/Saved Buddy/);
    expect(native.confirmImport).toHaveBeenCalledWith({
      displayName: "Abra",
      cp: "469",
      ivs: null,
      tagIds: [],
    });
  });
  const detected: ScreenshotPreview = {
    ...image,
    extraction: {
      ...image.extraction,
      attack: {
        value: 0,
        confidence: "tentative",
        method: "appraisal_geometry_v1",
        state: "detected",
      },
      defense: {
        value: 14,
        confidence: "tentative",
        method: "appraisal_geometry_v1",
        state: "detected",
      },
      hp: {
        value: 15,
        confidence: "tentative",
        method: "appraisal_geometry_v1",
        state: "detected",
      },
      issues: [],
    },
  };
  it("requires confirmation, permits corrections, retains them on save failure and sends only reviewed fields", async () => {
    vi.mocked(native.selectScreenshot).mockResolvedValue(detected);
    vi.mocked(native.confirmImport).mockRejectedValueOnce({
      code: "storage",
      message: "Try again.",
    });
    render(<Inventory />);
    await selectImage();
    expect(screen.getByLabelText("Attack IV")).toHaveValue("0");
    expect(screen.getByLabelText("HP IV")).toHaveValue("15");
    expect(screen.getByText(/Detected suggestion: 0/)).toBeInTheDocument();
    expect(screen.getByLabelText("CP (optional)")).toHaveValue("");
    expect(native.confirmImport).not.toHaveBeenCalled();
    fillName();
    fireEvent.change(screen.getByLabelText("Attack IV"), {
      target: { value: "1" },
    });
    expect(
      screen.getByText("Manually edited. Original suggestion: 0."),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Confirm and save" }));
    await screen.findByText("Try again.");
    expect(screen.getByLabelText("Attack IV")).toHaveValue("1");
    fireEvent.click(screen.getByRole("button", { name: "Confirm and save" }));
    await screen.findByText(/Saved Buddy/);
    expect(native.confirmImport).toHaveBeenLastCalledWith({
      displayName: "Buddy",
      cp: null,
      ivs: { attack: 1, defense: 14, hp: 15 },
      tagIds: [],
    });
  });
  it("clears old manual values and suggestions when replacing an image", async () => {
    vi.mocked(native.selectScreenshot)
      .mockResolvedValueOnce(detected)
      .mockResolvedValueOnce(image);
    render(<Inventory />);
    await selectImage();
    fillName();
    fireEvent.change(screen.getByLabelText("CP (optional)"), {
      target: { value: "123" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Replace screenshot" }));
    await screen.findByText(
      "Appraisal bars unresolved. Manual entry remains available.",
    );
    for (const label of [
      "Display name (required)",
      "CP (optional)",
      "Attack IV",
      "Defense IV",
      "HP IV",
    ]) {
      expect(screen.getByLabelText(label)).toHaveValue("");
    }
    expect(screen.queryByText(/Detected suggestion:/)).not.toBeInTheDocument();
    expect(native.confirmImport).not.toHaveBeenCalled();
    expect(revokeObjectURL).toHaveBeenCalledTimes(1);
  });
  it("keeps manual fallback usable when extraction is unresolved", async () => {
    render(<Inventory />);
    await selectImage();
    expect(
      screen.getByRole("list", { name: "Extraction notices" }),
    ).toHaveTextContent(/unresolved/);
    fillName();
    fireEvent.click(screen.getByRole("button", { name: "Confirm and save" }));
    await screen.findByText(/Saved Buddy/);
    expect(native.confirmImport).toHaveBeenCalledWith({
      displayName: "Buddy",
      cp: null,
      ivs: null,
      tagIds: [],
    });
  });
  it("discards detected values on cancel without persisting", async () => {
    vi.mocked(native.selectScreenshot).mockResolvedValue(detected);
    render(<Inventory />);
    await selectImage();
    fireEvent.click(screen.getByRole("button", { name: "Cancel review" }));
    expect(screen.queryByText(/Detected suggestion:/)).not.toBeInTheDocument();
    expect(native.confirmImport).not.toHaveBeenCalled();
    expect(revokeObjectURL).toHaveBeenCalledTimes(1);
  });
});
