import { invoke } from "@tauri-apps/api/core";

export interface AppraisalIvs {
  attack: number;
  defense: number;
  hp: number;
}
export interface Tag {
  id: string;
  name: string;
  scope: "standard" | "local";
}
export interface InventoryRecord {
  id: string;
  displayName: string;
  cp: string | null;
  ivs: AppraisalIvs | null;
  tags: Tag[];
}
export interface InventorySnapshot {
  records: InventoryRecord[];
  tags: Tag[];
}
export interface ScreenshotPreview {
  png: number[];
  width: number;
  height: number;
  extraction: ScreenshotExtraction;
}
export interface ExtractedField<T> {
  value: T | null;
  confidence: "none" | "tentative";
  method: "unavailable" | "appraisal_geometry_v1";
  state: "detected" | "unresolved" | "unavailable";
}
export interface ScreenshotExtraction {
  displayName: ExtractedField<string>;
  cp: ExtractedField<string>;
  attack: ExtractedField<number>;
  defense: ExtractedField<number>;
  hp: ExtractedField<number>;
  issues: { code: string; message: string }[];
}
export interface ImportInput {
  displayName: string;
  cp: string | null;
  ivs: AppraisalIvs | null;
  tagIds: string[];
}

export const native = {
  snapshot: () => invoke<InventorySnapshot>("inventory_snapshot"),
  selectScreenshot: () => invoke<ScreenshotPreview | null>("select_screenshot"),
  confirmImport: (input: ImportInput) =>
    invoke<InventoryRecord>("confirm_inventory_import", { input }),
};

export function nativeErrorMessage(error: unknown, fallback: string): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    typeof error.code === "string" &&
    "message" in error &&
    typeof error.message === "string"
  )
    return error.message;
  return fallback;
}
