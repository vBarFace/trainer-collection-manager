import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { IndividualDetails, type IndividualFacts } from "./IndividualDetails";
import type { InventoryRecord } from "./native";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const facts: IndividualFacts = {
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
beforeEach(() =>
  vi.mocked(invoke).mockReset().mockResolvedValue({
    values: facts,
    observationRevision: "7",
    needsReconfirmation: false,
    evidenceToken: "details-token",
  }),
);
afterEach(cleanup);
function mount() {
  const onSaved = vi.fn().mockResolvedValue(undefined);
  const onActiveChange = vi.fn();
  render(
    <IndividualDetails
      record={{ id: "00000000-0000-4000-8000-000000000001" } as InventoryRecord}
      disabled={false}
      onSaved={onSaved}
      onActiveChange={onActiveChange}
    />,
  );
  return { onSaved, onActiveChange };
}
it("keeps unknown variants distinct and submits only explicitly confirmed optional facts", async () => {
  const callbacks = mount();
  fireEvent.click(
    screen.getByRole("button", { name: "Review optional observations" }),
  );
  expect(await screen.findByLabelText("Shiny")).toHaveValue("");
  fireEvent.change(screen.getByLabelText("Current HP"), {
    target: { value: "47" },
  });
  fireEvent.change(screen.getByLabelText("Maximum HP"), {
    target: { value: "47" },
  });
  fireEvent.change(screen.getByLabelText("Favorite"), {
    target: { value: "true" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Confirm optional observations" }),
  );
  await screen.findByText(/Optional observations confirmed/);
  expect(invoke).toHaveBeenLastCalledWith("save_individual_details", {
    id: "00000000-0000-4000-8000-000000000001",
    expectedRevision: "7",
    expectedEvidenceToken: "details-token",
    values: { ...facts, currentHp: 47, maximumHp: 47, favorite: true },
  });
  expect(callbacks.onSaved).toHaveBeenCalledOnce();
});
it("rejects nonnumeric input before JSON could turn NaN into an unknown observation", async () => {
  mount();
  fireEvent.click(
    screen.getByRole("button", { name: "Review optional observations" }),
  );
  await screen.findByLabelText("Weight (kg)");
  fireEvent.change(screen.getByLabelText("Weight (kg)"), {
    target: { value: "unknown number" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Confirm optional observations" }),
  );
  expect(await screen.findByRole("alert")).toHaveTextContent(/valid numbers/);
  expect(invoke).toHaveBeenCalledTimes(1);
  fireEvent.click(
    screen.getByRole("button", { name: "Cancel optional observations" }),
  );
  expect(invoke).toHaveBeenCalledTimes(1);
});
