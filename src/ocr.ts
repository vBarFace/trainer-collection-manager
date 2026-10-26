import type { ScreenshotExtraction, ScreenshotPreview } from "./native";
import { textSuggestion, type OcrReading } from "./ocr-policy";

type Action =
  | "load"
  | "loadLanguage"
  | "initialize"
  | "setParameters"
  | "recognize";

// Narrow transport for the pinned Tesseract.js 6.0.1 worker protocol. Owning the
// raw Worker lets cancellation terminate it even during engine initialization;
// the public createWorker API exposes its handle only after initialization.
export async function extractText(
  image: ScreenshotPreview,
  signal: AbortSignal,
): Promise<ScreenshotExtraction> {
  const result = {
    ...image.extraction,
    issues: image.extraction.issues.filter(
      (issue) => issue.code !== "text_unavailable",
    ),
  };
  let worker: Worker | undefined;
  let pending:
    | { resolve: (value: unknown) => void; reject: (error: Error) => void }
    | undefined;
  let stopped = false;
  const stop = (reason: Error) => {
    stopped = true;
    worker?.terminate();
    pending?.reject(reason);
    pending = undefined;
  };
  const abort = () =>
    stop(new DOMException("Extraction cancelled", "AbortError"));
  const timeout = window.setTimeout(
    () => stop(new Error("OCR timeout")),
    30_000,
  );
  signal.addEventListener("abort", abort, { once: true });
  const unresolved = () =>
    ({
      value: null,
      confidence: "none",
      method: "tesseract_lstm",
      state: "unresolved",
    }) as const;
  result.displayName = unresolved();
  result.cp = unresolved();
  try {
    if (signal.aborted)
      throw new DOMException("Extraction cancelled", "AbortError");
    if (!image.textRegions.displayName && !image.textRegions.cp) return result;
    const base = new URL("/ocr/", window.location.href).href;
    worker = new Worker(`${base}worker.min.js`);
    let nextId = 0;
    worker.onmessage = ({
      data,
    }: MessageEvent<{ status: string; data: unknown }>) => {
      if (data.status === "progress" || !pending) return;
      const current = pending;
      pending = undefined;
      if (data.status === "resolve") current.resolve(data.data);
      else current.reject(new Error("Local OCR failed"));
    };
    worker.onerror = () => stop(new Error("Local OCR unavailable"));
    const request = <T>(action: Action, payload: object) =>
      new Promise<T>((resolve, reject) => {
        if (stopped || signal.aborted) {
          reject(new Error("OCR stopped"));
          return;
        }
        pending = { resolve: (data) => resolve(data as T), reject };
        worker!.postMessage({
          workerId: "local-review",
          jobId: `ocr-${++nextId}`,
          action,
          payload,
        });
      });
    await request("load", {
      options: { lstmOnly: true, corePath: base, logging: false },
    });
    await request("loadLanguage", {
      langs: ["eng", "por"],
      options: {
        langPath: `${base}lang`,
        cacheMethod: "none",
        gzip: true,
        lstmOnly: true,
      },
    });
    await request("initialize", {
      langs: ["eng", "por"],
      oem: 1,
      config: { load_system_dawg: "0", load_freq_dawg: "0" },
    });
    for (const key of ["displayName", "cp"] as const) {
      const png = image.textRegions[key];
      if (!png) continue;
      await request("setParameters", {
        params: {
          tessedit_pageseg_mode: "7",
          tessedit_char_whitelist: key === "cp" ? "CPcp0123456789" : "",
          user_defined_dpi: "150",
        },
      });
      const reading = await request<OcrReading>("recognize", {
        image: new Uint8Array(png),
        options: {},
        output: { text: true },
      });
      result[key] = textSuggestion(key, reading);
    }
  } catch {
    if (signal.aborted)
      throw new DOMException("Extraction cancelled", "AbortError");
    // Discard partial text results after a worker failure. Native IV suggestions
    // remain available; users can complete every field manually.
    result.displayName = unresolved();
    result.cp = unresolved();
    result.issues.push({
      code: "ocr_unavailable",
      message:
        "Local text recognition could not finish. Enter the name and visible CP manually.",
    });
  } finally {
    window.clearTimeout(timeout);
    signal.removeEventListener("abort", abort);
    worker?.terminate();
    pending = undefined;
    for (const key of ["displayName", "cp"] as const) {
      if (result[key].value === null && !signal.aborted)
        result.issues.push({
          code: `${key}_unresolved`,
          message:
            key === "cp"
              ? "CP was hidden or could not be read reliably. Leave it empty if unknown."
              : "The display name could not be read reliably. Enter it manually.",
        });
    }
  }
  return result;
}
