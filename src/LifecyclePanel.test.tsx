import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  EvolutionPanel,
  HistoryPanel,
  ReobservationPanel,
} from "./LifecyclePanel";
import { Inventory } from "./Inventory";
import {
  native,
  type HistoryEvent,
  type InventoryRecord,
  type ScreenshotPreview,
} from "./native";
import { extractText } from "./ocr";

vi.mock("./ocr", () => ({ extractText: vi.fn() }));
vi.mock("./native", async (original) => {
  const actual = await original<typeof import("./native")>();
  return {
    ...actual,
    native: {
      ...actual.native,
      snapshot: vi.fn(),
      selectScreenshot: vi.fn(),
      confirmImport: vi.fn(),
      evolutionOptions: vi.fn(),
      confirmEvolution: vi.fn(),
      reobserve: vi.fn(),
      history: vi.fn(),
    },
  };
});

const record: InventoryRecord = {
  id: "00000000-0000-4000-8000-000000000001",
  displayName: "Buddy",
  cp: "9007199254740993",
  strategicObservationRevision: "9007199254740994",
  ivs: { attack: 0, defense: 14, hp: 15 },
  tags: [{ id: "9", name: "SAVE/SPECIAL", scope: "standard" }],
  reference: {
    state: "resolved",
    identity: { species: "Charmander", form: "Normal" },
    dataset: {
      datasetId: "fictional-lifecycle-tests",
      version: "1",
      sourceUrl: "https://example.invalid/source",
      upstreamRevision: "test-1",
      retrievedAt: "2026-10-27",
      rightsEvidenceUrl: "https://example.invalid/rights",
      license: "MIT",
      notice: "Test-only relationship fixture; no dataset bundled.",
    },
    species: {
      id: "charmander",
      dexNumber: 4,
      name: "Charmander",
      aliases: [],
    },
    form: {
      id: "charmander-normal",
      speciesId: "charmander",
      name: "Normal",
      aliases: [],
      types: null,
      baseStats: null,
      evolutions: [{ targetFormId: "charmeleon-normal", requirements: null }],
    },
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
const options = {
  expectedStrategicObservationRevision: record.strategicObservationRevision,
  datasetId: record.reference.dataset.datasetId,
  datasetVersion: record.reference.dataset.version,
  targets: [
    { formId: "charmeleon-normal", species: "Charmeleon", form: "Normal" },
    { formId: "other-normal", species: "Other branch", form: "Normal" },
  ],
  unavailableReason: null,
};
const image: ScreenshotPreview = {
  png: [137, 80, 78, 71],
  width: 1080,
  height: 1920,
  textRegions: { displayName: null, cp: null },
  extraction: {
    displayName: {
      value: "Charmander",
      confidence: "tentative",
      method: "tesseract_lstm",
      engineConfidence: 74,
      state: "detected",
    },
    cp: {
      value: "9223372036854775807",
      confidence: "tentative",
      method: "tesseract_lstm",
      engineConfidence: 51,
      state: "detected",
    },
    attack: {
      value: 1,
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
const createObjectURL = vi.fn(() => "blob:lifecycle-preview");
const revokeObjectURL = vi.fn();
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
function props(value = record) {
  return {
    record: value,
    busy: false,
    onCommit: vi.fn(async (action: () => Promise<InventoryRecord>) => {
      await action();
    }),
    onActiveChange: vi.fn(),
    onReload: vi.fn().mockResolvedValue(undefined),
  };
}
async function startEvolution() {
  fireEvent.click(screen.getByRole("button", { name: "Evolve" }));
  await screen.findByLabelText("Evolution target");
}
function chooseEvolution() {
  fireEvent.change(screen.getByLabelText("Evolution target"), {
    target: { value: "charmeleon-normal" },
  });
  fireEvent.click(
    screen.getByLabelText("I confirm this evolution occurred in Pokémon GO"),
  );
}
async function startScreenshot() {
  fireEvent.click(
    screen.getByRole("button", { name: "Update from Screenshot" }),
  );
  await screen.findByLabelText("Observed CP");
}
function verifyIndividual() {
  fireEvent.click(
    screen.getByLabelText(
      "I verified this is the selected individual and its current species/form",
    ),
  );
}
function confirmObservation() {
  fireEvent.click(
    screen.getByRole("button", { name: "Confirm observation update" }),
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(native.evolutionOptions).mockResolvedValue(options);
  vi.mocked(native.confirmEvolution).mockResolvedValue({ ...record, cp: null });
  vi.mocked(native.reobserve).mockResolvedValue({ ...record, cp: "123" });
  vi.mocked(native.selectScreenshot).mockResolvedValue(image);
  vi.mocked(native.snapshot).mockResolvedValue({ records: [record], tags: [] });
  vi.mocked(native.history).mockResolvedValue([]);
  vi.mocked(extractText).mockImplementation(
    async (preview) => preview.extraction,
  );
  vi.stubGlobal(
    "URL",
    Object.assign(URL, { createObjectURL, revokeObjectURL }),
  );
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("confirmed evolution", () => {
  it("requires selecting a branch and confirming the actual action; cancellation writes nothing", async () => {
    const callbacks = props();
    render(<EvolutionPanel {...callbacks} />);
    await startEvolution();
    expect(screen.getByLabelText("Evolution target")).toHaveValue("");
    expect(native.confirmEvolution).not.toHaveBeenCalled();
    chooseEvolution();
    fireEvent.click(screen.getByRole("button", { name: "Cancel evolution" }));
    expect(native.confirmEvolution).not.toHaveBeenCalled();
    expect(callbacks.onActiveChange).toHaveBeenLastCalledWith(false);
  });

  it("sends the selected UUID, catalog context and exact expected revision only after confirmation", async () => {
    const callbacks = props();
    render(<EvolutionPanel {...callbacks} />);
    await startEvolution();
    chooseEvolution();
    fireEvent.click(screen.getByRole("button", { name: "Confirm evolution" }));
    await waitFor(() => expect(callbacks.onCommit).toHaveBeenCalledOnce());
    expect(native.confirmEvolution).toHaveBeenCalledExactlyOnceWith(record.id, {
      requestId: expect.any(String),
      expectedStrategicObservationRevision: record.strategicObservationRevision,
      datasetId: options.datasetId,
      datasetVersion: options.datasetVersion,
      targetFormId: "charmeleon-normal",
    });
  });

  it("explains unavailable reference data without guessing a target", async () => {
    vi.mocked(native.evolutionOptions).mockResolvedValue({
      ...options,
      targets: [],
      unavailableReason:
        "Reference data unavailable. No target can be validated.",
    });
    render(<EvolutionPanel {...props()} />);
    fireEvent.click(screen.getByRole("button", { name: "Evolve" }));
    expect(await screen.findByText(/No target can be validated/)).toBeVisible();
    expect(native.confirmEvolution).not.toHaveBeenCalled();
  });

  it("prevents repeat confirmation while saving and requires reloading a conflicting draft", async () => {
    const pending = deferred<InventoryRecord>();
    vi.mocked(native.confirmEvolution).mockReturnValue(pending.promise);
    render(<EvolutionPanel {...props()} />);
    await startEvolution();
    chooseEvolution();
    const confirm = screen.getByRole("button", { name: "Confirm evolution" });
    fireEvent.click(confirm);
    fireEvent.click(confirm);
    expect(native.confirmEvolution).toHaveBeenCalledOnce();
    await act(async () =>
      pending.reject({
        code: "conflict",
        message: "Reload the changed Pokémon.",
      }),
    );
    expect(
      await screen.findByText("Reload the changed Pokémon."),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Confirm evolution" }),
    ).toBeDisabled();
  });

  it("recovers an evolution whose committed reply was lost by reloading after already_confirmed", async () => {
    const callbacks = props();
    vi.mocked(native.confirmEvolution)
      .mockRejectedValueOnce(new Error("Reply lost"))
      .mockRejectedValueOnce({
        code: "already_confirmed",
        message: "This evolution is already confirmed. Reload the individual.",
      });
    render(<EvolutionPanel {...callbacks} />);
    await startEvolution();
    chooseEvolution();
    fireEvent.click(screen.getByRole("button", { name: "Confirm evolution" }));
    await screen.findByText(/Unable to save evolution/);
    const firstRequest = vi.mocked(native.confirmEvolution).mock.calls[0][1];
    fireEvent.click(screen.getByRole("button", { name: "Confirm evolution" }));
    await screen.findByText(/This evolution is already confirmed/);
    expect(native.confirmEvolution).toHaveBeenCalledTimes(2);
    expect(vi.mocked(native.confirmEvolution).mock.calls[1][1]).toEqual(
      firstRequest,
    );
    expect(
      screen.getByRole("button", { name: "Confirm evolution" }),
    ).toBeDisabled();
    fireEvent.click(
      screen.getByRole("button", { name: "Reload current individual" }),
    );
    await waitFor(() => expect(callbacks.onReload).toHaveBeenCalledOnce());
    await waitFor(() =>
      expect(
        screen.queryByLabelText("Evolution target"),
      ).not.toBeInTheDocument(),
    );
    expect(callbacks.onActiveChange).toHaveBeenLastCalledWith(false);
  });
});

describe("targeted screenshot re-observation", () => {
  it("shows tentative evidence and current values without applying suggestions automatically", async () => {
    render(<ReobservationPanel {...props()} />);
    await startScreenshot();
    expect(screen.getByLabelText("Observed CP")).toHaveValue(
      image.extraction.cp.value,
    );
    expect(screen.getByLabelText("Update CP")).not.toBeChecked();
    expect(screen.getByLabelText("Update appraisal IVs")).not.toBeChecked();
    expect(screen.getByText(new RegExp(record.cp!))).toBeVisible();
    expect(screen.getAllByText(/tentative/i).length).toBeGreaterThan(0);
    expect(native.reobserve).not.toHaveBeenCalled();
    expect(native.confirmImport).not.toHaveBeenCalled();
  });

  it("confirms an explicitly selected CP patch on the existing UUID and keeps the IV group", async () => {
    const callbacks = props();
    render(<ReobservationPanel {...callbacks} />);
    await startScreenshot();
    fireEvent.click(screen.getByLabelText("Update CP"));
    verifyIndividual();
    confirmObservation();
    await waitFor(() => expect(native.reobserve).toHaveBeenCalledOnce());
    expect(native.reobserve).toHaveBeenCalledExactlyOnceWith(record.id, {
      requestId: expect.any(String),
      expectedStrategicObservationRevision: record.strategicObservationRevision,
      observedIdentity: record.reference.identity,
      sameIndividualConfirmed: true,
      cp: { value: "9223372036854775807", source: "screenshot" },
      ivs: null,
    });
    expect(native.confirmImport).not.toHaveBeenCalled();
    await waitFor(() => expect(revokeObjectURL).toHaveBeenCalledOnce());
  });

  it("records edited values as manual instead of screenshot observations", async () => {
    render(<ReobservationPanel {...props()} />);
    await startScreenshot();
    fireEvent.click(screen.getByLabelText("Update CP"));
    fireEvent.change(screen.getByLabelText("Observed CP"), {
      target: { value: "123" },
    });
    fireEvent.click(screen.getByLabelText("Update appraisal IVs"));
    fireEvent.change(screen.getByLabelText("Observed attack IV"), {
      target: { value: "2" },
    });
    verifyIndividual();
    confirmObservation();
    await waitFor(() => expect(native.reobserve).toHaveBeenCalledOnce());
    expect(native.reobserve).toHaveBeenCalledWith(
      record.id,
      expect.objectContaining({
        cp: { value: "123", source: "manual" },
        ivs: {
          value: { attack: 2, defense: 14, hp: 15 },
          source: "manual",
        },
      }),
    );
  });

  it("keeps missing CP unknown after evolution while a reviewed IV patch is saved", async () => {
    vi.mocked(native.selectScreenshot).mockResolvedValue({
      ...image,
      extraction: {
        ...image.extraction,
        cp: {
          value: null,
          confidence: "none",
          method: "unavailable",
          state: "unavailable",
        },
      },
    });
    render(<ReobservationPanel {...props({ ...record, cp: null })} />);
    await startScreenshot();
    expect(screen.getByLabelText("Observed CP")).toHaveValue("");
    fireEvent.click(screen.getByLabelText("Update appraisal IVs"));
    verifyIndividual();
    confirmObservation();
    await waitFor(() => expect(native.reobserve).toHaveBeenCalledOnce());
    expect(native.reobserve).toHaveBeenCalledWith(
      record.id,
      expect.objectContaining({ cp: null }),
    );
  });

  it("surfaces a species mismatch and prevents interpreting it as an evolution", async () => {
    render(<ReobservationPanel {...props()} />);
    await startScreenshot();
    fireEvent.change(screen.getByLabelText("Observed species"), {
      target: { value: "Charmeleon" },
    });
    fireEvent.click(screen.getByLabelText("Update CP"));
    verifyIndividual();
    confirmObservation();
    expect(native.reobserve).not.toHaveBeenCalled();
    expect(native.confirmEvolution).not.toHaveBeenCalled();
    expect(
      screen.getAllByText(/mismatch|does not match/i).length,
    ).toBeGreaterThan(0);
  });

  it("cancels and releases the preview without writing", async () => {
    const callbacks = props();
    render(<ReobservationPanel {...callbacks} />);
    await startScreenshot();
    fireEvent.click(
      screen.getByRole("button", { name: "Cancel screenshot update" }),
    );
    expect(native.reobserve).not.toHaveBeenCalled();
    expect(native.confirmImport).not.toHaveBeenCalled();
    expect(revokeObjectURL).toHaveBeenCalledOnce();
    expect(callbacks.onActiveChange).toHaveBeenLastCalledWith(false);
  });

  it("aborts extraction and releases the preview when unmounted", async () => {
    const pending = deferred<ScreenshotPreview["extraction"]>();
    let signal: AbortSignal | undefined;
    vi.mocked(extractText).mockImplementation((_preview, currentSignal) => {
      signal = currentSignal;
      return pending.promise;
    });
    const view = render(<ReobservationPanel {...props()} />);
    fireEvent.click(
      screen.getByRole("button", { name: "Update from Screenshot" }),
    );
    await waitFor(() => expect(extractText).toHaveBeenCalledOnce());
    view.unmount();
    expect(signal?.aborted).toBe(true);
    expect(revokeObjectURL).toHaveBeenCalledOnce();
    await act(async () => pending.resolve(image.extraction));
    expect(native.reobserve).not.toHaveBeenCalled();
  });

  it("captures expectations before extraction and rejects duplicate submissions", async () => {
    const pending = deferred<InventoryRecord>();
    vi.mocked(native.reobserve).mockReturnValue(pending.promise);
    const callbacks = props();
    const view = render(<ReobservationPanel {...callbacks} />);
    await startScreenshot();
    view.rerender(
      <ReobservationPanel
        {...callbacks}
        record={{ ...record, strategicObservationRevision: "9007199254740995" }}
      />,
    );
    fireEvent.click(screen.getByLabelText("Update CP"));
    verifyIndividual();
    const confirm = screen.getByRole("button", {
      name: "Confirm observation update",
    });
    fireEvent.click(confirm);
    fireEvent.click(confirm);
    expect(native.reobserve).toHaveBeenCalledOnce();
    expect(native.reobserve).toHaveBeenCalledWith(
      record.id,
      expect.objectContaining({
        expectedStrategicObservationRevision:
          record.strategicObservationRevision,
      }),
    );
    await act(async () =>
      pending.reject({
        code: "conflict",
        message: "Reload current observations.",
      }),
    );
    expect(
      await screen.findByText("Reload current observations."),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Confirm observation update" }),
    ).toBeDisabled();
  });

  it("requires reload after retrying an observation update whose committed reply was lost", async () => {
    const callbacks = props();
    vi.mocked(native.reobserve)
      .mockRejectedValueOnce(new Error("Reply lost"))
      .mockRejectedValueOnce({
        code: "already_confirmed",
        message:
          "These observations are already confirmed. Reload the individual.",
      });
    render(<ReobservationPanel {...callbacks} />);
    await startScreenshot();
    fireEvent.click(screen.getByLabelText("Update CP"));
    verifyIndividual();
    confirmObservation();
    await screen.findByText(/Unable to save observations/);
    const firstRequest = vi.mocked(native.reobserve).mock.calls[0][1];
    confirmObservation();
    await screen.findByText(/These observations are already confirmed/);
    expect(native.reobserve).toHaveBeenCalledTimes(2);
    expect(vi.mocked(native.reobserve).mock.calls[1][1]).toEqual(firstRequest);
    expect(
      screen.getByRole("button", { name: "Confirm observation update" }),
    ).toBeDisabled();
    fireEvent.click(
      screen.getByRole("button", { name: "Reload current individual" }),
    );
    await waitFor(() => expect(callbacks.onReload).toHaveBeenCalledOnce());
    await waitFor(() =>
      expect(screen.queryByLabelText("Observed CP")).not.toBeInTheDocument(),
    );
    expect(revokeObjectURL).toHaveBeenCalledOnce();
    expect(callbacks.onActiveChange).toHaveBeenLastCalledWith(false);
  });

  it("updates the selected collection entry through Inventory without calling new import", async () => {
    render(<Inventory />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Review appraisal for Buddy" }),
    );
    await startScreenshot();
    fireEvent.click(screen.getByLabelText("Update CP"));
    verifyIndividual();
    confirmObservation();
    await waitFor(() => expect(native.reobserve).toHaveBeenCalledOnce());
    await screen.findByText("CP: 123");
    expect(
      screen.getAllByRole("button", { name: "Review appraisal for Buddy" }),
    ).toHaveLength(1);
    expect(native.confirmImport).not.toHaveBeenCalled();
  });
});

describe("confirmed history loading", () => {
  it.each(["success", "failure"])(
    "discards an old in-flight %s response after the selected record changes",
    async (outcome) => {
      const pending = deferred<HistoryEvent[]>();
      vi.mocked(native.history)
        .mockReturnValueOnce(pending.promise)
        .mockResolvedValueOnce([]);
      const view = render(<HistoryPanel record={record} />);
      fireEvent.click(
        screen.getByRole("button", { name: "Load confirmed history" }),
      );
      expect(native.history).toHaveBeenCalledExactlyOnceWith(record.id);
      view.rerender(
        <HistoryPanel
          record={{
            ...record,
            strategicObservationRevision: "9007199254740995",
          }}
        />,
      );
      await act(async () => {
        if (outcome === "failure")
          pending.reject({ code: "storage", message: "Old history error" });
        else
          pending.resolve([
            {
              requestId: "00000000-0000-4000-8000-000000000002",
              kind: "evolution",
              confirmedAt: "2088-01-01 00:00:00",
              pokemonId: record.id,
              observedIdentity: record.reference.identity,
              sameIndividualConfirmed: true,
              datasetId: record.reference.dataset.datasetId,
              datasetVersion: record.reference.dataset.version,
              before: {
                speciesId: "charmander",
                formId: "charmander-normal",
                identity: record.reference.identity,
                cp: "888",
                ivs: record.ivs,
              },
              after: {
                speciesId: "charmander",
                formId: "charmander-normal",
                identity: record.reference.identity,
                cp: null,
                ivs: record.ivs,
              },
              cpSource: null,
              ivsSource: null,
            },
          ]);
      });
      expect(screen.queryByText(/2088-01-01/)).not.toBeInTheDocument();
      expect(screen.queryByText("Old history error")).not.toBeInTheDocument();
      fireEvent.click(
        screen.getByRole("button", { name: "Load confirmed history" }),
      );
      expect(
        await screen.findByText("No confirmed lifecycle events yet."),
      ).toBeVisible();
      expect(native.history).toHaveBeenCalledTimes(2);
    },
  );
});
