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
