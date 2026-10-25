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

vi.mock("./native", async (importOriginal) => {
  const actual = await importOriginal<typeof import("./native")>();
  return {
    ...actual,
    native: {
      snapshot: vi.fn(),
      selectScreenshot: vi.fn(),
      confirmImport: vi.fn(),
    },
  };
});

const special = { id: "9", name: "SAVE/SPECIAL", scope: "standard" as const };
const custom = { id: "10", name: "Favorites", scope: "local" as const };
const image: ScreenshotPreview = {
  png: [137, 80, 78, 71],
  width: 1080,
  height: 1920,
};
const saved: InventoryRecord = {
  id: "00000000-0000-4000-8000-000000000000",
  displayName: "Buddy",
  cp: "9223372036854775807",
  ivs: { attack: 0, defense: 14, hp: 15 },
  tags: [special, custom],
};
const imported: InventoryRecord = { ...saved, tags: [] };
let createObjectURL: ReturnType<typeof vi.fn>;
let revokeObjectURL: ReturnType<typeof vi.fn>;

beforeEach(() => {
  vi.resetAllMocks();
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
  await screen.findByRole("heading", { name: "Review details manually" });
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
      screen.getByRole("heading", { name: "Awaiting evaluation" }),
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
