import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { PowerUpPanel } from "./PowerUpPanel";
import { HistoryPanel } from "./LifecyclePanel";
import { Inventory } from "./Inventory";
import { native, type InventoryRecord, type HistoryEvent } from "./native";

vi.mock("./native", async (original) => {
  const actual = await original<typeof import("./native")>();
  return {
    ...actual,
    native: {
      ...actual.native,
      confirmPowerUp: vi.fn(),
      snapshot: vi.fn(),
      history: vi.fn(),
      selectScreenshot: vi.fn(),
      confirmImport: vi.fn(),
      correctCp: vi.fn(),
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
function review(cp = "") {
  fireEvent.click(screen.getByRole("button", { name: "Power Up" }));
  fireEvent.change(screen.getByLabelText("Updated CP (optional)"), {
    target: { value: cp },
  });
  fireEvent.click(screen.getByRole("button", { name: "Review Power Up" }));
}
function confirm() {
  fireEvent.click(
    screen.getByLabelText("I confirm I performed this Power Up in Pokémon GO"),
  );
  fireEvent.click(screen.getByRole("button", { name: "Confirm Power Up" }));
}
function event(cp: string | null): HistoryEvent {
  const observation = {
    identity: record.reference.identity,
    speciesId: "charmander",
    formId: "charmander-normal",
    cp: record.cp,
    ivs: record.ivs,
  };
  return {
    requestId: "00000000-0000-4000-8000-000000000003",
    pokemonId: record.id,
    kind: "power_up",
    confirmedAt: "2026-10-27 12:00:00",
    before: observation,
    after: { ...observation, cp },
    cpSource: "manual",
    ivsSource: null,
    observedIdentity: null,
    sameIndividualConfirmed: true,
    datasetId: record.reference.dataset.datasetId,
    datasetVersion: record.reference.dataset.version,
  };
}
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(native.confirmPowerUp).mockResolvedValue({
    ...record,
    cp: "9223372036854775807",
  });
  vi.mocked(native.snapshot).mockResolvedValue({
    records: [record],
    tags: record.tags,
  });
  vi.mocked(native.history).mockResolvedValue([]);
});
afterEach(cleanup);

describe("confirmed Power Up", () => {
  it("previews exact CP difference and preserves the selected UUID until explicit confirmation", async () => {
    const callbacks = props();
    render(<PowerUpPanel {...callbacks} />);
    review("9223372036854775807");
    expect(screen.getByText(/UUID:/)).toHaveTextContent(record.id);
    expect(screen.getByText(/Species:/)).toHaveTextContent("Charmander");
    expect(
      screen.getByText("CP difference: +9214364837600034814"),
    ).toBeVisible();
    expect(native.confirmPowerUp).not.toHaveBeenCalled();
    expect(
      screen.getByRole("button", { name: "Confirm Power Up" }),
    ).toBeDisabled();
    confirm();
    await screen.findByText("Power Up confirmed and saved locally.");
    expect(screen.getByRole("button", { name: "Power Up" })).toHaveFocus();
    expect(native.confirmPowerUp).toHaveBeenCalledExactlyOnceWith(record.id, {
      requestId: expect.stringMatching(/^[0-9a-f-]{36}$/),
      expectedStrategicObservationRevision: record.strategicObservationRevision,
      cp: "9223372036854775807",
      performedInGameConfirmed: true,
    });
    expect(callbacks.onActiveChange.mock.calls).toEqual([[true], [false]]);
    expect(native.confirmImport).not.toHaveBeenCalled();
    expect(native.selectScreenshot).not.toHaveBeenCalled();
    expect(native.correctCp).not.toHaveBeenCalled();
  });
  it.each([null, "9007199254740993"])(
    "records no CP patch when previous CP is %s",
    async (cp) => {
      render(<PowerUpPanel {...props({ ...record, cp })} />);
      review();
      expect(screen.getByText(/CP after confirmation:/)).toHaveTextContent(
        cp ?? "Unknown",
      );
      expect(screen.queryByText(/CP difference:/)).not.toBeInTheDocument();
      confirm();
      await screen.findByText("Power Up confirmed and saved locally.");
      expect(vi.mocked(native.confirmPowerUp).mock.calls[0][1].cp).toBeNull();
    },
  );
  it("allows a manual CP observation when previous CP is unknown", async () => {
    render(<PowerUpPanel {...props({ ...record, cp: null })} />);
    review("1234");
    expect(screen.getByText("Saved CP: Unknown")).toBeVisible();
    expect(screen.queryByText(/CP difference:/)).not.toBeInTheDocument();
    confirm();
    await screen.findByText("Power Up confirmed and saved locally.");
    expect(vi.mocked(native.confirmPowerUp).mock.calls[0][1].cp).toBe("1234");
  });
  it.each(["entry", "preview"])(
    "cancels at %s without any persistence call",
    (phase) => {
      const callbacks = props();
      render(<PowerUpPanel {...callbacks} />);
      if (phase === "preview") review("1000");
      else fireEvent.click(screen.getByRole("button", { name: "Power Up" }));
      fireEvent.click(screen.getByRole("button", { name: "Cancel Power Up" }));
      expect(
        screen.getByText("Power Up cancelled. Nothing was saved."),
      ).toBeVisible();
      expect(screen.getByRole("button", { name: "Power Up" })).toHaveFocus();
      expect(native.confirmPowerUp).not.toHaveBeenCalled();
      expect(callbacks.onActiveChange).toHaveBeenLastCalledWith(false);
    },
  );
  it.each(["0", "-1", "1.5", "01", " 1", "9223372036854775808"])(
    "rejects invalid CP %s before preview",
    (cp) => {
      render(<PowerUpPanel {...props()} />);
      review(cp);
      expect(screen.getByRole("alert")).toHaveTextContent(
        "CP must be a whole number",
      );
      expect(
        screen.queryByRole("button", { name: "Confirm Power Up" }),
      ).not.toBeInTheDocument();
      expect(native.confirmPowerUp).not.toHaveBeenCalled();
    },
  );
  it("does not reuse confirmation after editing the preview", () => {
    render(<PowerUpPanel {...props()} />);
    review("1000");
    fireEvent.click(
      screen.getByLabelText(
        "I confirm I performed this Power Up in Pokémon GO",
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "Edit Power Up" }));
    fireEvent.change(screen.getByLabelText("Updated CP (optional)"), {
      target: { value: "2000" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Review Power Up" }));
    expect(
      screen.getByRole("button", { name: "Confirm Power Up" }),
    ).toBeDisabled();
    expect(native.confirmPowerUp).not.toHaveBeenCalled();
  });
  it("blocks rapid duplicate confirmation and cancellation while persistence is pending", async () => {
    let resolve!: (value: InventoryRecord) => void;
    vi.mocked(native.confirmPowerUp).mockReturnValue(
      new Promise((done) => {
        resolve = done;
      }),
    );
    render(<PowerUpPanel {...props()} />);
    review("1000");
    confirm();
    fireEvent.click(screen.getByRole("button", { name: "Confirm Power Up" }));
    expect(native.confirmPowerUp).toHaveBeenCalledOnce();
    expect(
      screen.getByRole("button", { name: "Cancel Power Up" }),
    ).toBeDisabled();
    await act(async () => resolve({ ...record, cp: "1000" }));
    expect(
      screen.getByText("Power Up confirmed and saved locally."),
    ).toBeVisible();
  });
  it("retains the exact request after a lost response and reloads an already confirmed action", async () => {
    const callbacks = props();
    vi.mocked(native.confirmPowerUp)
      .mockRejectedValueOnce({ code: "storage" })
      .mockRejectedValueOnce({
        code: "already_confirmed",
        message: "Already saved. Reload.",
      });
    render(<PowerUpPanel {...callbacks} />);
    review("1000");
    confirm();
    await screen.findByText(/Unable to save Power Up/);
    const input = vi.mocked(native.confirmPowerUp).mock.calls[0][1];
    fireEvent.click(screen.getByRole("button", { name: "Confirm Power Up" }));
    await screen.findByText("Already saved. Reload.");
    expect(vi.mocked(native.confirmPowerUp).mock.calls[1][1]).toEqual(input);
    expect(
      screen.getByRole("button", { name: "Confirm Power Up" }),
    ).toBeDisabled();
    fireEvent.click(
      screen.getByRole("button", { name: "Reload current individual" }),
    );
    await screen.findByText("Inventory reloaded. Start a new Power Up review.");
    expect(callbacks.onReload).toHaveBeenCalledOnce();
  });
  it.each(["conflict", "missing_pokemon"])(
    "requires reload after %s and preserves the draft on reload failure",
    async (code) => {
      const callbacks = props();
      callbacks.onReload.mockRejectedValueOnce({
        code: "storage",
        message: "Reload failed",
      });
      vi.mocked(native.confirmPowerUp).mockRejectedValueOnce({
        code,
        message: "Changed record",
      });
      render(<PowerUpPanel {...callbacks} />);
      review("1000");
      confirm();
      await screen.findByText("Changed record");
      expect(
        screen.getByRole("button", { name: "Confirm Power Up" }),
      ).toBeDisabled();
      fireEvent.click(
        screen.getByRole("button", { name: "Reload current individual" }),
      );
      await screen.findByText("Reload failed");
      expect(screen.getByText(/CP after confirmation: 1000/)).toBeVisible();
      expect(native.confirmPowerUp).toHaveBeenCalledOnce();
    },
  );
  it("locks other collection actions and refreshes authoritative data/history for the same individual", async () => {
    vi.mocked(native.history).mockResolvedValueOnce([
      event("9223372036854775807"),
    ]);
    render(<Inventory />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Review appraisal for Buddy" }),
    );
    review("9223372036854775807");
    expect(
      screen.getByRole("button", { name: "Review appraisal for Buddy" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Correct saved CP" }),
    ).toBeDisabled();
    expect(screen.getByRole("button", { name: "Evolve" })).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Update from Screenshot" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Select screenshot" }),
    ).toBeDisabled();
    confirm();
    await screen.findByText("Power Up confirmed and saved locally.");
    expect(screen.getByText("Confirmed CP: 9223372036854775807")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Review appraisal for Buddy" }),
    ).toBeEnabled();
    fireEvent.click(
      screen.getByRole("button", { name: "Load confirmed history" }),
    );
    await screen.findByText(/Confirmed Power Up.*2026-10-27/);
    expect(native.history).toHaveBeenCalledExactlyOnceWith(record.id);
    expect(native.confirmImport).not.toHaveBeenCalled();
  });
  it("displays Power Up separately from observation and evolution history and clears outdated history on update", async () => {
    vi.mocked(native.history).mockResolvedValue([
      event("1000"),
      { ...event("2000"), requestId: "evolution", kind: "evolution" },
      { ...event("3000"), requestId: "observation", kind: "reobservation" },
    ]);
    const view = render(<HistoryPanel record={record} />);
    fireEvent.click(
      screen.getByRole("button", { name: "Load confirmed history" }),
    );
    await screen.findByText(/Confirmed Power Up.*2026-10-27/);
    expect(screen.getByText(/Confirmed evolution.*2026-10-27/)).toBeVisible();
    expect(screen.getByText(/Updated observation.*2026-10-27/)).toBeVisible();
    view.rerender(<HistoryPanel record={{ ...record, cp: "1000" }} />);
    await waitFor(() =>
      expect(
        screen.queryByText(/Confirmed Power Up.*2026-10-27/),
      ).not.toBeInTheDocument(),
    );
  });
});
it("releases the collection lock when a missing individual disappears during reload", async () => {
  vi.mocked(native.confirmPowerUp).mockRejectedValueOnce({
    code: "missing_pokemon",
    message: "Individual deleted",
  });
  vi.mocked(native.snapshot)
    .mockResolvedValueOnce({ records: [record], tags: record.tags })
    .mockResolvedValueOnce({ records: [], tags: record.tags });
  render(<Inventory />);
  fireEvent.click(
    await screen.findByRole("button", { name: "Review appraisal for Buddy" }),
  );
  review();
  confirm();
  await screen.findByText("Individual deleted");
  fireEvent.click(
    screen.getByRole("button", { name: "Reload current individual" }),
  );
  await screen.findByText(
    "No Pokémon saved yet. Confirm a screenshot review to add one.",
  );
  expect(
    screen.getByRole("button", { name: "Select screenshot" }),
  ).toBeEnabled();
});
it("invalidates loaded history when an action preserves CP and the observation revision", async () => {
  vi.mocked(native.confirmPowerUp).mockResolvedValueOnce(
    structuredClone(record),
  );
  vi.mocked(native.history)
    .mockResolvedValueOnce([])
    .mockResolvedValueOnce([{ ...event(record.cp), cpSource: null }]);
  render(<Inventory />);
  fireEvent.click(
    await screen.findByRole("button", { name: "Review appraisal for Buddy" }),
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Load confirmed history" }),
  );
  await screen.findByText("No confirmed lifecycle events yet.");
  review();
  confirm();
  await screen.findByText("Power Up confirmed and saved locally.");
  expect(
    screen.queryByText("No confirmed lifecycle events yet."),
  ).not.toBeInTheDocument();
  fireEvent.click(
    screen.getByRole("button", { name: "Load confirmed history" }),
  );
  await screen.findByText(/Confirmed Power Up.*2026-10-27/);
  expect(native.history).toHaveBeenCalledTimes(2);
});
