// Build-time copies only. npm's lockfile pins the reviewed engine/language bytes.
// No network downloads, executable discovery or private screenshot inputs.
import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";

const root = new URL("../", import.meta.url);
const output = new URL("public/ocr/", root);
for (const [name, expected] of [
  ["tesseract.js", "6.0.1"],
  ["tesseract.js-core", "6.1.2"],
  ["@tesseract.js-data/eng", "1.0.0"],
  ["@tesseract.js-data/por", "1.0.0"],
]) {
  const installed = JSON.parse(
    await readFile(new URL(`node_modules/${name}/package.json`, root), "utf8"),
  );
  if (installed.version !== expected)
    throw new Error(
      `Review OCR protocol/assets/licenses before upgrading ${name}: expected ${expected}, got ${installed.version}`,
    );
}
await mkdir(new URL("lang/", output), { recursive: true });
const sources = [
  ["tesseract.js/dist/worker.min.js", "worker.min.js"],
  ["tesseract.js/dist/worker.min.js.LICENSE.txt", "worker.min.js.LICENSE.txt"],
  ...["", "-simd", "-lstm", "-simd-lstm"].map((variant) => [
    `tesseract.js-core/tesseract-core${variant}.wasm.js`,
    `tesseract-core${variant}.wasm.js`,
  ]),
  ...["eng", "por"].map((lang) => [
    `@tesseract.js-data/${lang}/4.0.0_best_int/${lang}.traineddata.gz`,
    `lang/${lang}.traineddata.gz`,
  ]),
  ["tesseract.js/LICENSE.md", "TESSERACT-JS-LICENSE.txt"],
  ["tesseract.js-core/LICENSE", "TESSERACT-CORE-LICENSE.txt"],
];
const manifest = [];
for (const [source, target] of sources) {
  const sourceUrl = new URL(`node_modules/${source}`, root);
  const bytes = await readFile(sourceUrl);
  await copyFile(sourceUrl, new URL(target, output));
  manifest.push({
    file: target,
    bytes: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  });
}
await copyFile(
  new URL("docs/legal/ocr-notices.txt", root),
  new URL("THIRD-PARTY-NOTICES.txt", output),
);
await copyFile(
  new URL("docs/legal/ocr-third-party-licenses.txt", root),
  new URL("OCR-THIRD-PARTY-LICENSES.txt", output),
);
await writeFile(
  new URL("manifest.json", output),
  `${JSON.stringify(manifest, null, 2)}\n`,
);
console.log(
  `Prepared ${manifest.length} local OCR assets in ${fileURLToPath(output)} (${manifest.reduce((sum, item) => sum + item.bytes, 0)} bytes).`,
);
