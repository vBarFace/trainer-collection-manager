import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("App", () => {
  it("renders accessible foundation navigation", () => {
    render(<App />);
    expect(
      screen.getByRole("navigation", { name: "Primary navigation" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Inventory" }),
    ).toBeInTheDocument();
    expect(screen.getByText("This area is a placeholder")).toBeInTheDocument();
  });
});
