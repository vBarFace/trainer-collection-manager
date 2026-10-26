import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ReferencePanel } from "./ReferencePanel";
import type { ReferenceResolution } from "./native";

afterEach(cleanup);

const reference: ReferenceResolution = {
  state: "not_set",
  identity: null,
  dataset: {
    datasetId: "original-fixture",
    version: "1",
    sourceUrl: "https://example.invalid/source",
    upstreamRevision: "fixture-1",
    retrievedAt: "2026-10-26",
    rightsEvidenceUrl: "https://example.invalid/license",
    license: "MIT",
    notice: "Synthetic test data; no game dataset bundled.",
  },
  species: null,
  form: null,
};
function panel(
  value = reference,
  onSave = vi.fn().mockResolvedValue(undefined),
  onReload = vi.fn(),
) {
  render(
    <ReferencePanel
      reference={value}
      busy={false}
      saving={false}
      onSave={onSave}
      onReload={onReload}
    />,
  );
  return { onSave, onReload };
}
function edit() {
  fireEvent.click(
    screen.getByRole("button", { name: "Correct species and form" }),
  );
}
function fill(species: string, form: string) {
  fireEvent.change(screen.getByLabelText("Species"), {
    target: { value: species },
  });
  fireEvent.change(screen.getByLabelText("Form"), { target: { value: form } });
}
function confirm() {
  fireEvent.click(
    screen.getByRole("button", { name: "Confirm identity correction" }),
  );
}

describe("reference details", () => {
  it.each([
    ["not_set", "Unresolved: no species or form has been confirmed."],
    ["unavailable", /Reference data unavailable/],
    ["unknown_species", /species is absent/],
    ["unknown_form", /No default form is assumed/],
    ["ambiguous", /matches multiple catalog entries/],
  ] as const)("explains %s without inventing metadata", (state, message) => {
    panel({ ...reference, state });
    expect(screen.getByText(message)).toBeVisible();
    expect(screen.queryByText(/National Pokédex/)).not.toBeInTheDocument();
    expect(screen.getByText(/Version: 1/)).toBeVisible();
  });

  it.each([["fire"], ["fire", "flying"]])(
    "shows available types %j and explicit missing metadata",
    (...types) => {
      panel({
        ...reference,
        state: "resolved",
        identity: { species: "Testling", form: "Plain" },
        species: {
          id: "testling",
          dexNumber: 9901,
          name: "Testling",
          aliases: [],
        },
        form: {
          id: "testling-plain",
          speciesId: "testling",
          name: "Plain",
          aliases: [],
          types,
          baseStats: null,
          evolutions: null,
        },
      });
      expect(screen.getByText(/National Pokédex: #9901/)).toBeVisible();
      expect(screen.getByText(`Types: ${types.join(" / ")}`)).toBeVisible();
      expect(screen.getByText("GO base stats: Unknown")).toBeVisible();
      expect(screen.getByText("Evolution information: Unknown")).toBeVisible();
    },
  );

  it("shows GO stats and available evolution requirements", () => {
    panel({
      ...reference,
      state: "resolved",
      form: {
        id: "testling-plain",
        speciesId: "testling",
        name: "Plain",
        aliases: [],
        types: null,
        baseStats: { attack: 101, defense: 102, stamina: 103 },
        evolutions: [
          { targetFormId: "testling-next", requirements: ["25 test candies"] },
        ],
      },
    });
    expect(
      screen.getByText(/101 attack \/ 102 defense \/ 103 stamina/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Target form: testling-next. Requirements: 25 test candies/,
      ),
    ).toBeVisible();
    expect(screen.getByText("Types: Unknown")).toBeVisible();
  });

  it("saves only explicit identity and preserves exact entered values offline", async () => {
    const network = vi
      .spyOn(globalThis, "fetch")
      .mockRejectedValue(new Error("offline"));
    try {
      const { onSave } = panel();
      edit();
      expect(screen.getByLabelText("Species")).toHaveValue("");
      expect(screen.getByLabelText("Species")).toHaveFocus();
      fill("Testling", "Plain");
      confirm();
      await screen.findByText("Species and form confirmation saved locally.");
      expect(onSave).toHaveBeenCalledExactlyOnceWith(null, {
        species: "Testling",
        form: "Plain",
      });
      expect(network).not.toHaveBeenCalled();
      expect(
        screen.getByRole("button", { name: "Correct species and form" }),
      ).toHaveFocus();
    } finally {
      network.mockRestore();
    }
  });

  it("keeps unknown form separate from cleared identity", async () => {
    const { onSave } = panel();
    edit();
    fill("Testling", "");
    confirm();
    await screen.findByText("Species and form confirmation saved locally.");
    expect(onSave).toHaveBeenLastCalledWith(null, {
      species: "Testling",
      form: null,
    });
    edit();
    confirm();
    await waitFor(() => expect(onSave).toHaveBeenLastCalledWith(null, null));
  });

  it.each([
    ["", "Plain"],
    [" Testling", ""],
    ["Testling", "Plain "],
    ["x".repeat(101), ""],
  ])("rejects invalid input %j / %j", (species, form) => {
    const { onSave } = panel();
    edit();
    fill(species, form);
    confirm();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Enter a species before a form.",
    );
    expect(onSave).not.toHaveBeenCalled();
  });

  it("retains a failed draft and compares the original identity on retry", async () => {
    const onSave = vi
      .fn()
      .mockRejectedValueOnce(new Error("unavailable"))
      .mockResolvedValue(undefined);
    const original = { species: "Previous", form: null };
    panel({ ...reference, identity: original }, onSave);
    edit();
    fill("Testling", "Plain");
    confirm();
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Your draft is retained",
    );
    expect(screen.getByLabelText("Species")).toHaveValue("Testling");
    confirm();
    await screen.findByText("Species and form confirmation saved locally.");
    expect(onSave).toHaveBeenNthCalledWith(2, original, {
      species: "Testling",
      form: "Plain",
    });
  });

  it("requires reload after a conflict and allows cancellation without saving", async () => {
    const { onSave, onReload } = panel(
      reference,
      vi.fn().mockRejectedValue({
        code: "conflict",
        message: "Identity changed. Reload.",
      }),
    );
    edit();
    fill("Testling", "Plain");
    confirm();
    await screen.findByText("Identity changed. Reload.");
    expect(
      screen.getByRole("button", { name: "Confirm identity correction" }),
    ).toBeDisabled();
    fireEvent.click(
      screen.getByRole("button", {
        name: "Discard identity draft and reload inventory",
      }),
    );
    expect(onReload).toHaveBeenCalledOnce();
    expect(onSave).toHaveBeenCalledOnce();
    edit();
    fill("Different", "");
    fireEvent.click(
      screen.getByRole("button", { name: "Cancel identity correction" }),
    );
    expect(
      screen.getByText("Identity correction cancelled. Nothing was saved."),
    ).toBeVisible();
    expect(onSave).toHaveBeenCalledOnce();
  });
});
