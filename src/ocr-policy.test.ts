import { describe, expect, it } from "vitest";
import { textSuggestion } from "./ocr-policy";

describe("OCR evidence admission", () => {
  it("accepts visible names without a species dictionary and preserves engine provenance", () => {
    for (const text of [
      "Abra",
      "Pokémon",
      "Meu Abra 2",
      "Nidoran♀",
      "Farfetch’d",
    ])
      expect(
        textSuggestion("displayName", { text: `${text}\n`, confidence: 91 }),
      ).toMatchObject({
        value: text,
        method: "tesseract_lstm",
        engineConfidence: 91,
      });
  });
  it("requires one complete CP-prefixed integer and keeps hidden or ambiguous CP unknown", () => {
    expect(
      textSuggestion("cp", { text: "CP468\n", confidence: 93 }).value,
    ).toBe("468");
    for (const text of [
      "468",
      "CP",
      "CP4?8",
      "CP46B",
      "CP468\nCP469",
      "CP0",
      "CP9223372036854775808",
    ])
      expect(textSuggestion("cp", { text, confidence: 99 }).value).toBeNull();
  });
  it("does not turn low confidence, contradictory or invalid text into a suggestion", () => {
    for (const confidence of [0, 59, NaN, Infinity, 101])
      expect(
        textSuggestion("displayName", { text: "Abra", confidence }).value,
      ).toBeNull();
    for (const text of [
      "",
      "123",
      "Abra\nKadabra",
      "Abra /",
      "a\u0000b",
      "x".repeat(101),
    ])
      expect(
        textSuggestion("displayName", { text, confidence: 99 }).value,
      ).toBeNull();
  });
});
