import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { GymAssessmentView } from "./GymAssessmentView";
import type { GymAssessment } from "./gym-native";

const assessment: GymAssessment = {
  state: "limited",
  rulesetVersion: "fictional-gym-1",
  summary: "Limited gym-attack potential under the fictional reference model.",
  pokemonTypes: ["normal"],
  vulnerableTo: ["fighting"],
  builds: [
    {
      attackType: "fire",
      fastMove: "Fictional Spark",
      chargedMove: "Fictional Burst",
      moveState: "hypothetical",
      neutralCycleDps: 12.34,
      referencePercentile: 72.5,
      referenceSummary:
        "Middle range of imported reference options in the neutral model.",
      comparedForms: 10,
      referenceBest: ["Fictional Reference A", "Fictional Reference B"],
      strongAgainst: ["grass", "ice"],
      resistedBy: ["water"],
    },
  ],
  missing: ["Current level remains unknown"],
  limitations: ["The same hypothetical level is used for both sides."],
  ownedAlternativePokemonId: null,
};
afterEach(cleanup);

it("leads with broad attack types, distinct Pokémon typing and limitations", () => {
  render(<GymAssessmentView assessment={assessment} />);
  expect(
    screen.getByRole("heading", { name: "Gym attack potential" }),
  ).toBeVisible();
  expect(screen.getByText(/Pokémon typing: Normal/)).toBeVisible();
  expect(
    screen.getByText(/Middle range of imported reference options/),
  ).toBeVisible();
  expect(
    screen.getByRole("heading", { name: "Fire attack potential" }),
  ).toBeVisible();
  expect(
    screen.getByText("Favorable type coverage: Grass, Ice."),
  ).toBeVisible();
  expect(screen.getByText("Reduced coverage against: Water.")).toBeVisible();
  expect(screen.getByText(/Dual typing/)).toBeVisible();
  expect(screen.getByText("Current level remains unknown")).toBeVisible();
  expect(
    screen.getByText("The same hypothetical level is used for both sides."),
  ).not.toBeVisible();
  expect(
    screen.getByText(
      /Gym defense and Team Rocket performance are not assessed/,
    ),
  ).toBeVisible();
});

it("keeps cycle estimates and external benchmark examples in optional details", () => {
  render(<GymAssessmentView assessment={assessment} />);
  const number = screen.getByText(/12.34 modeled damage per second/);
  const detail = number.closest("details")!;
  expect(detail).not.toHaveAttribute("open");
  expect(number).not.toBeVisible();
  fireEvent.click(screen.getByText("Reference-model details for Fire"));
  // jsdom does not implement native summary activation, so use its native
  // open state to verify the disclosure rather than simulate application state.
  detail.open = true;
  expect(number).toBeVisible();
  expect(screen.getByText(/Reference percentile: 72.5%/)).toBeVisible();
  expect(
    screen.getByText(/Fictional Reference A, Fictional Reference B/),
  ).toBeVisible();
  expect(screen.getByText(/not a battle simulation/)).toBeVisible();
});

it("separates proposed moves from confirmed move observations", () => {
  const view = render(<GymAssessmentView assessment={assessment} />);
  expect(
    screen.getByText(/These are proposed attacks, not confirmed observations/),
  ).toBeVisible();
  view.rerender(
    <GymAssessmentView
      assessment={{
        ...assessment,
        builds: [{ ...assessment.builds[0], moveState: "confirmed" }],
      }}
    />,
  );
  expect(
    screen.getByText(/Move names match your confirmed observations/),
  ).toBeVisible();
  expect(
    screen.queryByText(/These are proposed attacks/),
  ).not.toBeInTheDocument();
});

it("opens an owned alternative without changing the external assessment", () => {
  const open = vi.fn();
  render(
    <GymAssessmentView
      assessment={{ ...assessment, ownedAlternativePokemonId: "other-uuid" }}
      onSelectPokemon={open}
    />,
  );
  fireEvent.click(
    screen.getByRole("button", {
      name: "Inspect owned gym-attack alternative",
    }),
  );
  expect(open).toHaveBeenCalledExactlyOnceWith("other-uuid");
  expect(screen.getByText(assessment.summary)).toBeVisible();
  expect(
    screen.getByText(
      /does not change this reference-based potential assessment/,
    ),
  ).toBeVisible();
});

it("withholds builds and benchmark claims when evidence is unavailable", () => {
  render(
    <GymAssessmentView
      assessment={{
        ...assessment,
        state: "needs_data",
        summary: "Install compatible combat references to assess gym attack.",
        builds: [],
      }}
    />,
  );
  expect(
    screen.getByText(/No supported attack build can be assessed/),
  ).toBeVisible();
  expect(screen.queryByText(/Reference percentile/)).not.toBeInTheDocument();
  expect(
    screen.queryByRole("heading", { name: "Fire attack potential" }),
  ).not.toBeInTheDocument();
});
