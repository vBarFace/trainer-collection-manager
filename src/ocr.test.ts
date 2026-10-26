import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { extractText } from "./ocr";
import type { ScreenshotPreview } from "./native";

const missing = {
  value: null,
  confidence: "none",
  method: "unavailable",
  state: "unavailable",
} as const;
const image: ScreenshotPreview = {
  png: [],
  width: 738,
  height: 1600,
  textRegions: { displayName: [1, 2], cp: [3, 4] },
  extraction: {
    displayName: missing,
    cp: missing,
    attack: {
      value: 15,
      confidence: "tentative",
      method: "appraisal_geometry_v1",
      state: "detected",
    },
    defense: {
      value: 2,
      confidence: "tentative",
      method: "appraisal_geometry_v1",
      state: "detected",
    },
    hp: {
      value: 2,
      confidence: "tentative",
      method: "appraisal_geometry_v1",
      state: "detected",
    },
    issues: [{ code: "text_unavailable", message: "Old native notice" }],
  },
};
let instances: FakeWorker[];
let hang: boolean;
let fail: boolean;
class FakeWorker {
  onmessage?: (event: { data: { status: string; data: unknown } }) => void;
  onerror?: () => void;
  terminated = false;
  cp = false;
  packets: { action: string; payload: Record<string, unknown> }[] = [];
  constructor(public url: string) {
    instances.push(this);
  }
  terminate() {
    this.terminated = true;
  }
  postMessage(packet: { action: string; payload: Record<string, unknown> }) {
    this.packets.push(packet);
    if (hang) return;
    if (packet.action === "setParameters")
      this.cp = Boolean(
        (packet.payload.params as Record<string, string>)
          .tessedit_char_whitelist,
      );
    queueMicrotask(() =>
      this.onmessage?.({
        data: {
          status: fail ? "reject" : "resolve",
          data:
            packet.action === "recognize"
              ? { text: this.cp ? "CP468\n" : "Abra\n", confidence: 96 }
              : {},
        },
      }),
    );
  }
}

beforeEach(() => {
  instances = [];
  hang = false;
  fail = false;
  vi.stubGlobal("Worker", FakeWorker);
});
afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe("bounded offline OCR lifecycle", () => {
  it("uses only bundled resources with caching disabled and emits typed editable suggestions", async () => {
    const result = await extractText(image, new AbortController().signal);
    expect(result.displayName.value).toBe("Abra");
    expect(result.cp.value).toBe("468");
    expect(result.attack.value).toBe(15);
    expect(result.issues).toEqual([]);
    const worker = instances[0];
    expect(new URL(worker.url).origin).toBe(window.location.origin);
    expect(
      worker.packets.find((packet) => packet.action === "loadLanguage")
        ?.payload,
    ).toMatchObject({
      options: {
        cacheMethod: "none",
        langPath: `${window.location.origin}/ocr/lang`,
      },
    });
    expect(worker.terminated).toBe(true);
  });
  it("terminates immediately even if cancelled during initialization", async () => {
    hang = true;
    const controller = new AbortController();
    const promise = extractText(image, controller.signal);
    controller.abort();
    await expect(promise).rejects.toMatchObject({ name: "AbortError" });
    expect(instances[0].terminated).toBe(true);
  });
  it("times out an unresponsive engine and keeps manual fallback and native IVs", async () => {
    vi.useFakeTimers();
    hang = true;
    const promise = extractText(image, new AbortController().signal);
    await vi.advanceTimersByTimeAsync(30_000);
    const result = await promise;
    expect(result.cp.value).toBeNull();
    expect(result.attack.value).toBe(15);
    expect(result.issues).toContainEqual(
      expect.objectContaining({ code: "ocr_unavailable" }),
    );
    expect(instances[0].terminated).toBe(true);
  });
  it("handles missing/corrupt engine assets without retaining a worker", async () => {
    fail = true;
    const result = await extractText(image, new AbortController().signal);
    expect(result.displayName.value).toBeNull();
    expect(result.cp.value).toBeNull();
    expect(instances[0].terminated).toBe(true);
  });
});
