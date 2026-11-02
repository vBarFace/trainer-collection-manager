import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { PreparationPlannerView } from "./PreparationPlannerView";
import { preparationNative, type PreparationPlan } from "./preparation-native";

vi.mock("./preparation-native", () => ({
  preparationNative: {
    resources: vi.fn(),
    saveResources: vi.fn(),
    selectPath: vi.fn(),
  },
}));
const id = "00000000-0000-4000-8000-000000000001";
const plan: PreparationPlan = {
  selectedPathId: null,
  state: "conditional",
  summary: "Compare future Fire coverage with retaining the current form.",
  currentLevel: null,
  resources: {
    revision: "2",
    candy: null,
    candyXl: 0,
    stardust: 10000,
    maximumLevel: null,
    normalCostsConfirmed: false,
  },
  paths: [
    {
      id: "evolve-spark",
      targetFormId: "fictional-spark",
      targetName: "Fictional Spark",
      targetLevel: 30,
      role: "gym_attack",
      actions: [
        {
          kind: "evolve",
          description: "Evolve the confirmed individual.",
          requirements: ["Fire Stone", "Observed evolution eligibility"],
          acquisition: ["Obtain Sample Flame through an Elite Charged TM"],
        },
      ],
      cost: { candy: 50, candyXl: 0, stardust: null, items: ["Fire Stone"] },
      benefit: "Adds Fire attack coverage against Grass and Ice types.",
      blockers: ["Confirm the current level"],
      affordability: "unknown",
      executable: false,
    },
  ],
  missing: ["Current level"],
  limitations: ["The current form remains a separate PvP option."],
};
beforeEach(() => {
  vi.resetAllMocks();
});
afterEach(cleanup);

it("distinguishes known zero costs from unknown costs and routes a conditional path to the same individual", () => {
  const open = vi.fn();
  render(
    <PreparationPlannerView
      pokemonId={id}
      plan={plan}
      onOpenIndividual={open}
      roles={[
        {
          role: "transfer",
          state: "held",
          summary: "Retain pending review of other roles.",
          reasons: ["Potential evolved coverage"],
          missing: ["PvP move performance"],
        },
      ]}
    />,
  );
  const option = screen.getByRole("article", {
    name: "Preparation option for Fictional Spark at level 30",
  });
  expect(
    within(option).getByText(/Candy: 50 · Candy XL: 0 · Stardust: Unknown/),
  ).toBeVisible();
  expect(
    within(option).getByText(/How to obtain:.*Elite Charged TM/),
  ).toBeVisible();
  expect(
    within(option).getByText(
      /Requirements: Fire Stone; Observed evolution eligibility/,
    ),
  ).toBeVisible();
  expect(screen.getByText(/Retain pending review/)).toBeVisible();
  expect(screen.getByText(/Confirmed current level: Unknown/)).toBeVisible();
  fireEvent.click(
    within(option).getByRole("button", {
      name: "Review facts and actions for this option",
    }),
  );
  expect(open).toHaveBeenCalledExactlyOnceWith(id);
});

it("saves explicit allocations without converting unknown values to zero or recording spending", async () => {
  const changed = vi.fn();
  vi.mocked(preparationNative.saveResources).mockResolvedValue({
    ...plan.resources!,
    revision: "3",
    candy: 0,
    normalCostsConfirmed: true,
  });
  render(
    <PreparationPlannerView
      pokemonId={id}
      plan={plan}
      onOpenIndividual={vi.fn()}
      onChanged={changed}
    />,
  );
  screen
    .getByText("Optional planning resources for this individual")
    .closest("details")!.open = true;
  expect(screen.getByLabelText("Candy allocation")).toHaveValue("");
  expect(screen.getByLabelText("Candy XL allocation")).toHaveValue("0");
  fireEvent.change(screen.getByLabelText("Candy allocation"), {
    target: { value: "0" },
  });
  fireEvent.click(screen.getByRole("checkbox"));
  fireEvent.click(
    screen.getByRole("button", { name: "Confirm planning allocation" }),
  );
  await screen.findByText(/Planning resources saved/);
  expect(preparationNative.saveResources).toHaveBeenCalledExactlyOnceWith(
    id,
    "2",
    {
      candy: 0,
      candyXl: 0,
      stardust: 10000,
      maximumLevel: null,
      normalCostsConfirmed: true,
      selectedPathId: null,
    },
  );
  expect(changed).toHaveBeenCalledOnce();
});

it("rejects invalid allocations locally and withholds actions on a stale saved path", async () => {
  const rendered = render(
    <PreparationPlannerView
      pokemonId={id}
      plan={plan}
      onOpenIndividual={vi.fn()}
    />,
  );
  screen
    .getByText("Optional planning resources for this individual")
    .closest("details")!.open = true;
  fireEvent.change(screen.getByLabelText("Candy allocation"), {
    target: { value: "-1" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Confirm planning allocation" }),
  );
  expect(screen.getByRole("alert")).toHaveTextContent(
    "Use whole resource amounts",
  );
  expect(preparationNative.saveResources).not.toHaveBeenCalled();
  rendered.rerender(
    <PreparationPlannerView
      pokemonId={id}
      plan={plan}
      onOpenIndividual={vi.fn()}
      stale
    />,
  );
  expect(
    screen.getByRole("button", {
      name: "Review facts and actions for this option",
    }),
  ).toBeDisabled();
  expect(
    screen.queryByText("Optional planning resources for this individual"),
  ).not.toBeInTheDocument();
});

it("keeps a conflicted allocation draft and never presents it as saved", async () => {
  vi.mocked(preparationNative.saveResources).mockRejectedValue({
    code: "conflict",
    message: "Allocation revision changed",
  });
  render(
    <PreparationPlannerView
      pokemonId={id}
      plan={plan}
      onOpenIndividual={vi.fn()}
    />,
  );
  screen
    .getByText("Optional planning resources for this individual")
    .closest("details")!.open = true;
  fireEvent.change(screen.getByLabelText("Candy allocation"), {
    target: { value: "25" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Confirm planning allocation" }),
  );
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Allocation revision changed",
  );
  expect(screen.getByLabelText("Candy allocation")).toHaveValue("25");
  expect(
    screen.queryByText(/Planning resources saved/),
  ).not.toBeInTheDocument();
});

it("persists the chosen roadmap path against the current context without reporting a game action", async () => {
  const changed = vi.fn();
  const open = vi.fn();
  vi.mocked(preparationNative.selectPath).mockResolvedValue({
    ...plan.resources!,
    revision: "3",
    selectedPathId: "evolve-spark",
  });
  render(
    <PreparationPlannerView
      pokemonId={id}
      plan={plan}
      expectedContext={"c".repeat(64)}
      onOpenIndividual={open}
      onChanged={changed}
    />,
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Use this plan in roadmap" }),
  );
  await screen.findByText("Selected roadmap plan");
  expect(preparationNative.selectPath).toHaveBeenCalledExactlyOnceWith(
    id,
    "c".repeat(64),
    "evolve-spark",
  );
  expect(changed).toHaveBeenCalledOnce();
  expect(open).not.toHaveBeenCalled();
  expect(
    screen.getByRole("button", { name: "Use this plan in roadmap" }),
  ).toBeDisabled();
});

it("shows other roles for an evolution and blocks stale path selection", () => {
  render(
    <PreparationPlannerView
      pokemonId={id}
      expectedContext={"c".repeat(64)}
      stale
      plan={{
        ...plan,
        paths: [
          {
            ...plan.paths[0],
            futureRoles: [
              {
                role: "gym_defense",
                state: "conditional",
                summary: "The evolved form retains potential defensive bulk.",
                reasons: ["External bulk comparison"],
                missing: ["Placement eligibility"],
              },
            ],
          },
        ],
      }}
      onOpenIndividual={vi.fn()}
    />,
  );
  screen.getByText("Other roles after preparation").closest("details")!.open =
    true;
  expect(
    screen.getByText("The evolved form retains potential defensive bulk."),
  ).toBeVisible();
  expect(screen.getByText("Still needed: Placement eligibility")).toBeVisible();
  expect(
    screen.getByRole("button", { name: "Use this plan in roadmap" }),
  ).toBeDisabled();
  expect(preparationNative.selectPath).not.toHaveBeenCalled();
});
