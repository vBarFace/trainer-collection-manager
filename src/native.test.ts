import { expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { native } from "./native";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

it("transports CP correction expectations and native results as exact decimal strings", async () => {
  const id = "00000000-0000-4000-8000-000000000000";
  const returned = {
    id,
    cp: "9223372036854775807",
    strategicObservationRevision: "9007199254740994",
  };
  vi.mocked(invoke).mockResolvedValue(returned);
  expect(
    await native.correctCp(
      id,
      "9007199254740993",
      "9007199254740993",
      "9223372036854775807",
    ),
  ).toBe(returned);
  expect(invoke).toHaveBeenLastCalledWith("correct_pokemon_cp", {
    id,
    input: {
      expectedCp: "9007199254740993",
      expectedStrategicObservationRevision: "9007199254740993",
      cp: "9223372036854775807",
    },
  });
  await native.correctCp(
    id,
    returned.cp,
    returned.strategicObservationRevision,
    null,
  );
  expect(invoke).toHaveBeenLastCalledWith("correct_pokemon_cp", {
    id,
    input: {
      expectedCp: returned.cp,
      expectedStrategicObservationRevision:
        returned.strategicObservationRevision,
      cp: null,
    },
  });
});

it("keeps lifecycle confirmation and targeted observation IPC separate from new imports", async () => {
  const id = "00000000-0000-4000-8000-000000000000";
  const requestId = "00000000-0000-4000-8000-000000000001";
  const evolution = {
    requestId,
    expectedStrategicObservationRevision: "9007199254740993",
    datasetId: "fixture",
    datasetVersion: "1",
    targetFormId: "next-normal",
  };
  await native.evolutionOptions(id);
  expect(invoke).toHaveBeenLastCalledWith("evolution_options", { id });
  await native.confirmEvolution(id, evolution);
  expect(invoke).toHaveBeenLastCalledWith("confirm_evolution", {
    id,
    input: evolution,
  });
  const observation = {
    requestId,
    expectedStrategicObservationRevision: "9007199254740995",
    cp: { value: "9223372036854775807", source: "screenshot" as const },
    ivs: null,
    observedIdentity: { species: "Next", form: "Normal" },
    sameIndividualConfirmed: true,
  };
  await native.reobserve(id, observation);
  expect(invoke).toHaveBeenLastCalledWith("reobserve_pokemon", {
    id,
    input: observation,
  });
  await native.history(id);
  expect(invoke).toHaveBeenLastCalledWith("pokemon_history", { id });
});
