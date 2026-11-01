import { invoke } from "@tauri-apps/api/core";
import type { ScreenshotPreview } from "./native";

export interface BatchSelection {
  sessionId: string;
  count: number;
}

export const batchNative = {
  select: () => invoke<BatchSelection | null>("select_screenshot_batch"),
  preview: (sessionId: string, index: number) =>
    invoke<ScreenshotPreview>("screenshot_batch_preview", { sessionId, index }),
  release: (sessionId: string) =>
    invoke<void>("release_screenshot_batch", { sessionId }),
};
