import type { ThemeColors } from "./theme";

export type { ThemeColors };

export type Action =
  | { type: "none" }
  | { type: "open"; path: string }
  | { type: "web"; query: string }
  | { type: "runShell"; command: string; flavor: string }
  | { type: "openTerminal"; command: string; flavor: string }
  | { type: "copy"; text: string }
  | { type: "showGraph" }
  | { type: "sleep" };

export type Row = {
  id: string;
  kind: string;
  title: string;
  subtitle: string;
  action: Action;
};

export type SeriesMeta = {
  label: string;
  expression: string;
  color: string;
  plot: string;
  domain?: string;
  style?: "line" | "dashed" | "points";
};

export type PlotMarker = {
  x: string;
  y: string;
};

export type PlotFrame = {
  xMin: string;
  xMax: string;
  yMin: string;
  yMax: string;
};

export type MathPayload = {
  largeValue: string | null;
  autoGraph: boolean;
  interpretationLatex: string;
  resultLabel: string | null;
  resultLatex: string | null;
  note: string | null;
  series: SeriesMeta[];
  markers: PlotMarker[];
  frame: PlotFrame | null;
};

export type QueryResponse = {
  mode: string;
  rows: Row[];
  math: MathPayload | null;
};

export type ActivateResponse = {
  output: string | null;
  hide: boolean;
  timedOut: boolean;
};

export type SettingsResponse = {
  hotkey: string;
  searchUrl: string;
  theme: string;
  colors: ThemeColors;
  anchor: string;
  xPercent: number;
  yPercent: number;
  hotkeyError: string | null;
  configPath: string;
};

export type SampleResponse = {
  series: { expression: string; lines: number[][][] }[];
};
