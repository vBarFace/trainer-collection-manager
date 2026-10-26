// Explicit local diagnostic, not part of app runtime. Never copies a screenshot,
// writes OCR crops or downloads language data. Private input remains outside Git.
import { execFileSync } from "node:child_process";
import { resolve } from "node:path";
import { createWorker, PSM } from "tesseract.js";
import { textSuggestion } from "../src/ocr-policy.ts";

if (process.argv.length !== 3)
  throw new Error(
    "Usage: node scripts/verify-local-extraction.mjs <authorized-local-image>",
  );
const output = execFileSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "inspect_local_extraction_for_ocr",
    "--",
    "--ignored",
    "--nocapture",
  ],
  {
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
    timeout: 120_000,
    env: { ...process.env, TCM_REFERENCE_SCREENSHOT: resolve(process.argv[2]) },
  },
);
const match = /^TCM_OCR_INPUT=(.+)$/m.exec(output);
if (!match)
  throw new Error("Native extraction diagnostic did not return inputs");
const input = JSON.parse(match[1]);
const started = performance.now();
const worker = await createWorker(
  ["eng", "por"],
  1,
  { langPath: resolve("public/ocr/lang"), cacheMethod: "none", gzip: true },
  { load_system_dawg: "0", load_freq_dawg: "0" },
);
try {
  const result = { ...input.extraction };
  for (const key of ["displayName", "cp"]) {
    if (!input.textRegions[key]) continue;
    await worker.setParameters({
      tessedit_pageseg_mode: PSM.SINGLE_LINE,
      tessedit_char_whitelist: key === "cp" ? "CPcp0123456789" : "",
      user_defined_dpi: "150",
    });
    const { data } = await worker.recognize(
      Buffer.from(input.textRegions[key]),
    );
    result[key] = textSuggestion(key, data);
  }
  console.log(
    JSON.stringify(
      {
        displayName: result.displayName,
        cp: result.cp,
        attack: result.attack,
        defense: result.defense,
        hp: result.hp,
        ocrMs: Math.round(performance.now() - started),
      },
      null,
      2,
    ),
  );
} finally {
  await worker.terminate();
}
