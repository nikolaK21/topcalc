import { invoke } from "@tauri-apps/api/core";
import type { Action, ActivateResponse, QueryResponse, SampleResponse, SettingsResponse } from "./types";

export function runQuery(input: string, mode = "decimal"): Promise<QueryResponse> {
  return invoke("query", { input, mode });
}

export function activate(action: Action): Promise<ActivateResponse> {
  return invoke("activate", { action });
}

export function sampleGraph(request: {
  expressions: string[];
  jobs?: { expression: string; plot: string; domain?: string }[];
  bindings?: { name: string; value: number }[];
  xMin: number;
  xMax: number;
  yMin?: number;
  yMax?: number;
  samples: number;
  degrees?: boolean;
}): Promise<SampleResponse> {
  return invoke("sample_graph", { request });
}

export function evalGraph(
  expression: string,
  bindings: { name: string; value: number }[],
  degrees: boolean,
  mode = "decimal",
): Promise<string> {
  return invoke("eval_graph", { expression, bindings, degrees, mode });
}

export function getSettings(): Promise<SettingsResponse> {
  return invoke("get_settings");
}

export function saveSettings(settings: {
  hotkey: string;
  searchUrl: string;
  theme: string;
  colors: SettingsResponse["colors"];
  anchor: string;
  xPercent: number;
  yPercent: number;
}): Promise<SettingsResponse> {
  return invoke("save_settings", { settings });
}

export function placeWindow(anchor: string, xPercent: number, yPercent: number): Promise<void> {
  return invoke("place_window", { anchor, xPercent, yPercent });
}

const DESK_QUEUE = "topcalc-desk-queue";

export function openDesk(expression: string, commit: boolean): Promise<void> {
  if (inTauri()) return invoke("open_desk", { expression, commit });
  const queued = JSON.parse(localStorage.getItem(DESK_QUEUE) || "[]") as { expression: string; commit: boolean }[];
  queued.push({ expression, commit });
  localStorage.setItem(DESK_QUEUE, JSON.stringify(queued));
  window.open(`${location.origin}${location.pathname}?desk=1`, "topcalc-desk");
  return Promise.resolve();
}

export function takeDeskLines(): Promise<{ expression: string; commit: boolean }[]> {
  return invoke("take_desk_lines");
}

export function suspendHotkey(): Promise<void> {
  return invoke("suspend_hotkey");
}

export function resumeHotkey(): Promise<void> {
  return invoke("resume_hotkey");
}

export function hideWindow(): Promise<void> {
  return invoke("hide_window");
}

export function quitApp(): Promise<void> {
  return invoke("quit");
}

export function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
