import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { BatchImport } from "./BatchImport";
import { batchNative } from "./batch-native";
import {
  native,
  type InventoryRecord,
  type ScreenshotExtraction,
  type ScreenshotPreview,
} from "./native";
import { extractText } from "./ocr";

vi.mock("./batch-native", () => ({
  batchNative: { select: vi.fn(), preview: vi.fn(), release: vi.fn() },
}));
vi.mock("./ocr", () => ({ extractText: vi.fn() }));
vi.mock("./native", async (original) => {
  const actual = await original<typeof import("./native")>();
  return {
    ...actual,
    native: { ...actual.native, confirmBatchImport: vi.fn() },
  };
});
const unknown = {
  value: null,
  confidence: "none",
  method: "unavailable",
  state: "unavailable",
} as const;
const extraction: ScreenshotExtraction = {
  displayName: {
    value: "Fictional friend",
    confidence: "tentative",
    method: "tesseract_lstm",
    state: "detected",
  },
  cp: { ...unknown },
  attack: { ...unknown },
  defense: { ...unknown },
  hp: { ...unknown },
  issues: [],
};
const image: ScreenshotPreview = {
  png: [137, 80, 78, 71],
  width: 12,
  height: 24,
  extraction,
  textRegions: { cp: null, displayName: null },
};
beforeEach(() => {
  vi.clearAllMocks();
  vi.stubGlobal("URL", {
    createObjectURL: vi.fn().mockReturnValue("blob:batch-preview"),
    revokeObjectURL: vi.fn(),
  });
  vi.mocked(batchNative.select).mockResolvedValue({
    sessionId: "session-1",
    count: 3,
  });
  vi.mocked(batchNative.preview).mockResolvedValue(image);
  vi.mocked(batchNative.release).mockResolvedValue(undefined);
  vi.mocked(extractText).mockResolvedValue(extraction);
  vi.mocked(native.confirmBatchImport).mockResolvedValue({
    id: "new-uuid",
  } as InventoryRecord);
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

async function open() {
  fireEvent.click(
    screen.getByRole("button", { name: "Select screenshot batch" }),
  );
  await screen.findByRole("button", { name: "Review screenshot 1" });
}
async function inspect(index = 1) {
  fireEvent.click(
    screen.getByRole("button", { name: `Review screenshot ${index}` }),
  );
  await screen.findByLabelText("Display name (required)");
  await waitFor(() =>
    expect(screen.getByLabelText("Display name (required)")).toBeEnabled(),
  );
}
function confirm() {
  fireEvent.click(
    screen.getByLabelText(
      "I reviewed these facts and confirm this is a new individual",
    ),
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Confirm batch individual" }),
  );
}

describe("bounded batch screenshot review", () => {
  it("extracts the entire queue sequentially without saving or retaining preview URLs", async () => {
    let concurrent = 0;
    let peak = 0;
    vi.mocked(extractText).mockImplementation(async () => {
      concurrent++;
      peak = Math.max(peak, concurrent);
      await Promise.resolve();
      concurrent--;
      return extraction;
    });
    render(<BatchImport onSaved={vi.fn()} />);
    await open();
    fireEvent.click(
      screen.getByRole("button", { name: "Extract queued screenshots" }),
    );
    await screen.findByText(
      /Local extraction finished: 3 screenshots processed/,
    );
    expect(vi.mocked(batchNative.preview).mock.calls).toEqual([
      ["session-1", 0],
      ["session-1", 1],
      ["session-1", 2],
    ]);
    expect(peak).toBe(1);
    expect(screen.getAllByText("Missing optional information")).toHaveLength(3);
    expect(URL.createObjectURL).not.toHaveBeenCalled();
    expect(native.confirmBatchImport).not.toHaveBeenCalled();
  });

  it("groups explicitly selected unsaved screenshots into one confirmed individual and clears conflicting facts", async () => {
    vi.mocked(extractText)
      .mockResolvedValueOnce({
        ...extraction,
        cp: {
          value: "100",
          confidence: "tentative",
          method: "tesseract_lstm",
          state: "detected",
        },
      })
      .mockResolvedValueOnce({
        ...extraction,
        cp: {
          value: "200",
          confidence: "tentative",
          method: "tesseract_lstm",
          state: "detected",
        },
      });
    render(<BatchImport onSaved={vi.fn()} />);
    await open();
    fireEvent.click(
      screen.getByRole("button", { name: "Extract queued screenshots" }),
    );
    await screen.findByText(
      /Local extraction finished: 3 screenshots processed/,
    );
    fireEvent.click(screen.getByLabelText("Group screenshot 1"));
    fireEvent.click(screen.getByLabelText("Group screenshot 2"));
    fireEvent.click(
      screen.getByRole("button", {
        name: "Group selected screenshots as one individual",
      }),
    );
    expect(screen.getByText("Grouped")).toBeVisible();
    expect(screen.getByLabelText("CP (optional)")).toHaveValue("");
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Conflicting CP values were cleared",
    );
    expect(native.confirmBatchImport).not.toHaveBeenCalled();
    fireEvent.click(
      screen.getByLabelText(
        "I reviewed every grouped screenshot and confirm they show the same new individual",
      ),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Confirm batch individual" }),
    );
    await waitFor(() => expect(screen.getAllByText("Saved")).toHaveLength(2));
    expect(native.confirmBatchImport).toHaveBeenCalledOnce();
    expect(
      screen.getByRole("button", { name: "Review screenshot 1" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Review screenshot 2" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Review screenshot 3" }),
    ).toBeEnabled();
  });

  it("queues many selections without decoding or saving, then reviews/corrects one and creates one new individual", async () => {
    const onSaved = vi.fn();
    render(<BatchImport onSaved={onSaved} />);
    await open();
    expect(batchNative.preview).not.toHaveBeenCalled();
    expect(native.confirmBatchImport).not.toHaveBeenCalled();
    expect(screen.getAllByText("Queued")).toHaveLength(3);
    await inspect();
    expect(batchNative.preview).toHaveBeenCalledExactlyOnceWith("session-1", 0);
    expect(screen.getByText("Missing optional information")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Confirm batch individual" }),
    ).toBeDisabled();
    fireEvent.change(screen.getByLabelText("Display name (required)"), {
      target: { value: "Corrected name" },
    });
    expect(screen.getByText("Manually entered")).toBeVisible();
    confirm();
    await screen.findByText("Saved");
    expect(native.confirmBatchImport).toHaveBeenCalledExactlyOnceWith(
      expect.stringMatching(/^[0-9a-f-]{36}$/),
      {
        displayName: "Corrected name",
        cp: null,
        ivs: null,
        tagIds: [],
      },
    );
    expect(onSaved).toHaveBeenCalledOnce();
    expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:batch-preview");
    expect(
      screen.getByRole("button", { name: "Review screenshot 2" }),
    ).toBeEnabled();
  });

  it("retains manual drafts across queue navigation and resets confirmation on edits", async () => {
    render(<BatchImport onSaved={vi.fn()} />);
    await open();
    await inspect();
    fireEvent.change(screen.getByLabelText("CP (optional)"), {
      target: { value: "123" },
    });
    fireEvent.click(
      screen.getByLabelText(
        "I reviewed these facts and confirm this is a new individual",
      ),
    );
    fireEvent.change(screen.getByLabelText("Display name (required)"), {
      target: { value: "Manual name" },
    });
    expect(
      screen.getByRole("button", { name: "Confirm batch individual" }),
    ).toBeDisabled();
    await inspect(2);
    await inspect(1);
    expect(screen.getByLabelText("CP (optional)")).toHaveValue("123");
    expect(screen.getByLabelText("Display name (required)")).toHaveValue(
      "Manual name",
    );
    expect(extractText).toHaveBeenCalledTimes(2);
  });

  it("allows manual completion after a failed decode and skipping does not save", async () => {
    vi.mocked(batchNative.preview).mockRejectedValueOnce({
      code: "image_read",
      message: "Cannot read image",
    });
    render(<BatchImport onSaved={vi.fn()} />);
    await open();
    await inspect();
    expect(screen.getByText("Extraction failed")).toBeVisible();
    expect(screen.getByRole("alert")).toHaveTextContent("Cannot read image");
    fireEvent.click(screen.getByRole("button", { name: "Skip screenshot" }));
    expect(screen.getByText("Skipped")).toBeVisible();
    expect(native.confirmBatchImport).not.toHaveBeenCalled();
    await inspect(2);
    confirm();
    await screen.findByText("Saved");
    expect(native.confirmBatchImport).toHaveBeenCalledOnce();
  });

  it("terminates extraction and releases queue/preview on cancellation without saving", async () => {
    let observedSignal: AbortSignal | undefined;
    vi.mocked(extractText).mockImplementationOnce(
      (_image, signal) =>
        new Promise((_resolve, reject) => {
          observedSignal = signal;
          signal.addEventListener(
            "abort",
            () => reject(new DOMException("Cancelled", "AbortError")),
            { once: true },
          );
        }),
    );
    const active = vi.fn();
    render(<BatchImport onSaved={vi.fn()} onActiveChange={active} />);
    await open();
    fireEvent.click(
      screen.getByRole("button", { name: "Review screenshot 1" }),
    );
    await screen.findByText("Processing locally…");
    await waitFor(() => expect(observedSignal).toBeDefined());
    fireEvent.click(
      screen.getByRole("button", { name: "Close batch / cancel extraction" }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Select screenshot batch" }),
      ).toBeEnabled(),
    );
    expect(observedSignal?.aborted).toBe(true);
    expect(batchNative.release).toHaveBeenCalledExactlyOnceWith("session-1");
    expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:batch-preview");
    expect(active).toHaveBeenLastCalledWith(false);
    expect(native.confirmBatchImport).not.toHaveBeenCalled();
  });

  it("keeps drafts after save failures and prevents a second pending submission", async () => {
    let done!: (record: InventoryRecord) => void;
    vi.mocked(native.confirmBatchImport).mockRejectedValueOnce({
      code: "storage",
      message: "Local save failed",
    });
    render(<BatchImport onSaved={vi.fn()} />);
    await open();
    await inspect();
    confirm();
    await screen.findByText("Local save failed");
    expect(screen.getByLabelText("Display name (required)")).toHaveValue(
      "Fictional friend",
    );
    vi.mocked(native.confirmBatchImport).mockReturnValueOnce(
      new Promise((resolve) => {
        done = resolve;
      }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Confirm batch individual" }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Confirm batch individual" }),
    );
    expect(native.confirmBatchImport).toHaveBeenCalledTimes(2);
    expect(vi.mocked(native.confirmBatchImport).mock.calls[1][0]).toBe(
      vi.mocked(native.confirmBatchImport).mock.calls[0][0],
    );
    expect(
      screen.getByRole("button", { name: "Close batch / cancel extraction" }),
    ).toBeDisabled();
    await act(async () => done({ id: "new-uuid" } as InventoryRecord));
    await screen.findByText("Saved");
  });

  it("releases a late picker result after unmount", async () => {
    let select!: (value: { sessionId: string; count: number }) => void;
    vi.mocked(batchNative.select).mockReturnValueOnce(
      new Promise((resolve) => {
        select = resolve;
      }),
    );
    const view = render(<BatchImport onSaved={vi.fn()} />);
    fireEvent.click(
      screen.getByRole("button", { name: "Select screenshot batch" }),
    );
    view.unmount();
    await act(async () => select({ sessionId: "late-session", count: 500 }));
    expect(batchNative.release).toHaveBeenCalledWith("late-session");
    expect(native.confirmBatchImport).not.toHaveBeenCalled();
  });
});
