import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ReferenceSettings } from "./ReferenceSettings";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() =>
  vi
    .mocked(invoke)
    .mockReset()
    .mockResolvedValue({ active: null, canRollback: false }),
);
afterEach(cleanup);

it("exports structured data only after an explicit user action", async () => {
  render(<ReferenceSettings />);
  await screen.findByText(/No user reference installed/);
  expect(invoke).toHaveBeenCalledTimes(1);
  vi.mocked(invoke).mockResolvedValueOnce(true);
  fireEvent.click(
    screen.getByRole("button", {
      name: "Export structured inventory and plans",
    }),
  );
  await screen.findByText(/exported without screenshots/);
  expect(invoke).toHaveBeenLastCalledWith("export_structured_backup");
});

it("requires an explicit source revision and keeps cancelled imports offline", async () => {
  render(<ReferenceSettings />);
  await screen.findByText(/No user reference installed/);
  const install = screen.getByRole("button", {
    name: "Select and activate reference JSON",
  });
  expect(install).toBeDisabled();
  fireEvent.change(screen.getByLabelText(/Upstream commit/), {
    target: { value: "local-source-2026-11-01" },
  });
  vi.mocked(invoke).mockResolvedValueOnce(null);
  fireEvent.click(install);
  await screen.findByText(/Import cancelled/);
  expect(invoke).toHaveBeenLastCalledWith("import_reference", {
    provider: "pvpoke",
    revision: "local-source-2026-11-01",
  });
  expect(screen.getByText(/No user reference installed/)).toBeVisible();
});

it("retains the active reference and offers retry after malformed source input", async () => {
  render(<ReferenceSettings />);
  await screen.findByText(/No user reference installed/);
  fireEvent.change(screen.getByLabelText(/Upstream commit/), {
    target: { value: "revision" },
  });
  vi.mocked(invoke).mockRejectedValueOnce({
    code: "reference_invalid",
    message: "Malformed source",
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Select and activate reference JSON" }),
  );
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Malformed source",
  );
  expect(
    screen.getByRole("button", { name: "Select and activate reference JSON" }),
  ).toBeEnabled();
  expect(
    screen.getByRole("button", { name: "Roll back to previous reference" }),
  ).toBeDisabled();
});
