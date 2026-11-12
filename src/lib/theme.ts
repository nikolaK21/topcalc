export type ThemeColors = {
  background: string;
  text: string;
  muted: string;
  accent: string;
  surface: string;
};

export const THEME_OPTIONS = [
  { id: "dark", label: "Dark" },
  { id: "light", label: "Light" },
  { id: "slate", label: "Slate" },
  { id: "forest", label: "Forest" },
  { id: "custom", label: "Custom" },
] as const;

export const ANCHOR_OPTIONS = [
  { id: "top", label: "Top" },
  { id: "center", label: "Center" },
  { id: "bottom", label: "Bottom" },
  { id: "top-left", label: "Top left" },
  { id: "top-right", label: "Top right" },
  { id: "bottom-left", label: "Bottom left" },
  { id: "bottom-right", label: "Bottom right" },
  { id: "custom", label: "Custom" },
] as const;

const DARK: ThemeColors = {
  background: "#1c1c20",
  text: "#f3f4f6",
  muted: "#8b93a7",
  accent: "#6ea8fe",
  surface: "#12141a",
};

const PRESETS: Record<string, ThemeColors> = {
  dark: DARK,
  light: {
    background: "#f3f4f6",
    text: "#1c1c20",
    muted: "#5c6570",
    accent: "#2563eb",
    surface: "#ffffff",
  },
  slate: {
    background: "#0f172a",
    text: "#e2e8f0",
    muted: "#94a3b8",
    accent: "#38bdf8",
    surface: "#1e293b",
  },
  forest: {
    background: "#13241c",
    text: "#e7f6ee",
    muted: "#8eaea0",
    accent: "#3dd68c",
    surface: "#0c1b14",
  },
};

export function presetColors(theme: string): ThemeColors {
  return { ...(PRESETS[theme] ?? DARK) };
}

export function cssVars(colors: ThemeColors): string {
  return `--background:${colors.background};--text:${colors.text};--muted:${colors.muted};--accent:${colors.accent};--surface:${colors.surface}`;
}

export function isLightBackground(hex: string): boolean {
  return luminance(hex) > 0.6;
}

export function luminance(hex: string): number {
  const body = hex.trim().replace("#", "");
  if (body.length !== 6) return 0;
  const channels = [0, 2, 4].map((index) => {
    const value = Number.parseInt(body.slice(index, index + 2), 16);
    const channel = value / 255;
    return channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

export function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 50;
  return Math.max(0, Math.min(100, Math.round(value)));
}
