// Optional real-worker diagnostic. Requires a locally installed Playwright and
// browser; neither is downloaded. Never persists the private image or crops.
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { createServer } from "vite";

if (process.argv.length !== 3)
  throw new Error(
    "Usage: node scripts/verify-browser-ocr.mjs <authorized-local-image>",
  );
const { chromium } = await import(
  process.env.TCM_PLAYWRIGHT_MODULE
    ? pathToFileURL(process.env.TCM_PLAYWRIGHT_MODULE).href
    : "playwright"
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
const input = JSON.parse(/^TCM_OCR_INPUT=(.+)$/m.exec(output)[1]);
const csp = JSON.parse(await readFile("src-tauri/tauri.conf.json", "utf8")).app
  .security.csp;
const server = await createServer({
  server: { host: "127.0.0.1", port: 1421, strictPort: true },
  clearScreen: false,
});
let browser;
try {
  await server.listen();
  browser = await chromium.launch({
    headless: true,
    ...(process.env.TCM_BROWSER_EXECUTABLE
      ? { executablePath: process.env.TCM_BROWSER_EXECUTABLE }
      : {}),
  });
  const context = await browser.newContext();
  const requests = [];
  const blocked = [];
  context.on("request", (request) => requests.push(request.url()));
  await context.route("**/*", (route) => {
    if (new URL(route.request().url()).origin === "http://127.0.0.1:1421")
      return route.continue();
    blocked.push(route.request().url());
    return route.abort();
  });
  const page = await context.newPage();
  await page.goto("http://127.0.0.1:1421");
  await page.evaluate((policy) => {
    const meta = document.createElement("meta");
    meta.httpEquiv = "Content-Security-Policy";
    meta.content = policy;
    document.head.append(meta);
  }, csp);
  const result = await page.evaluate(async (input) => {
    const { extractText } = await import("/src/ocr.ts");
    const started = performance.now();
    const result = await extractText(input, new AbortController().signal);
    return {
      result,
      elapsedMs: Math.round(performance.now() - started),
      databases: await indexedDB.databases(),
    };
  }, input);
  if (blocked.length)
    throw new Error(`Unexpected remote resource request (${blocked.length})`);
  const expected = process.env.TCM_EXPECTED_EXTRACTION
    ? JSON.parse(process.env.TCM_EXPECTED_EXTRACTION)
    : null;
  if (
    expected &&
    Object.entries(expected).some(
      ([field, value]) => result.result[field]?.value !== value,
    )
  )
    throw new Error(
      `Reference extraction mismatch: ${JSON.stringify(result.result)}`,
    );
  console.log(
    JSON.stringify(
      {
        displayName: result.result.displayName,
        cp: result.result.cp,
        ivs: [
          result.result.attack.value,
          result.result.defense.value,
          result.result.hp.value,
        ],
        elapsedMs: result.elapsedMs,
        remoteRequests: blocked.length,
        indexedDbDatabases: result.databases.length,
        localOcrRequests: requests.filter((url) => url.includes("/ocr/")),
      },
      null,
      2,
    ),
  );
} finally {
  await browser?.close();
  await server.close();
}
