import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";

vi.mock("./native", () => ({
  native: { snapshot: vi.fn().mockResolvedValue({ records: [], tags: [] }) },
}));
describe("App", () => {
  it("renders accessible inventory navigation", async () => {
    render(<App />);
    expect(
      screen.getByRole("navigation", { name: "Primary navigation" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Inventory" }),
    ).toBeInTheDocument();
    expect(
      await screen.findByText("No saved Pokémon yet."),
    ).toBeInTheDocument();
  });
});
