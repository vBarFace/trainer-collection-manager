import type { ExtractedField } from "./native";

export interface OcrReading {
  text: string;
  confidence: number;
}

// Engine confidence is an uncalibrated Tesseract score, not an accuracy percent.
// These are conservative admission thresholds, pending a labelled benchmark.
export function textSuggestion(
  kind: "displayName" | "cp",
  reading: OcrReading,
): ExtractedField<string> {
  const text = reading.text.trim().normalize("NFC");
  let value: string | null = null;
  if (
    Number.isFinite(reading.confidence) &&
    reading.confidence >= 60 &&
    reading.confidence <= 100
  ) {
    if (kind === "cp") {
      const match = /^CP\s*([1-9][0-9]{0,18})$/i.exec(text);
      if (match && BigInt(match[1]) <= 9223372036854775807n) value = match[1];
    } else if (
      Array.from(text).length <= 100 &&
      /\p{L}/u.test(text) &&
      /^[\p{L}\p{M}\p{N} .’'♀♂-]+$/u.test(text) &&
      !/[\r\n]/u.test(text)
    ) {
      value = text;
    }
  }
  return {
    value,
    method: "tesseract_lstm",
    state: value === null ? "unresolved" : "detected",
    confidence: value === null ? "none" : "tentative",
    ...(Number.isFinite(reading.confidence) &&
    reading.confidence >= 0 &&
    reading.confidence <= 100
      ? { engineConfidence: reading.confidence }
      : {}),
  };
}
