export interface RivetPaletteTokens {
  bg: string;
  panel: string;
  panelSoft: string;
  text: string;
  muted: string;
  accent: string;
  ok: string;
  danger: string;
  border: string;
}

export interface RivetThemeTokens {
  radius: {
    panel: number;
  };
  shadows: {
    dayPanel: string;
    nightPanel: string;
  };
  typography: {
    bodyFontFamily: string;
    monoFontFamily: string;
  };
  day: RivetPaletteTokens;
  night: RivetPaletteTokens;
}

export const RIVET_TOKENS: RivetThemeTokens = {
  radius: {
    panel: 14
  },
  shadows: {
    dayPanel: "0 8px 22px rgba(29, 17, 8, 0.05)",
    nightPanel: "0 10px 28px rgba(0, 0, 0, 0.35)"
  },
  typography: {
    bodyFontFamily: "\"Source Sans 3\", \"Segoe UI\", sans-serif",
    monoFontFamily: "\"Source Code Pro\", monospace"
  },
  day: {
    bg: "#f4efe7",
    panel: "#fffdf7",
    panelSoft: "#f7f0e4",
    text: "#241d17",
    muted: "#6c5d4f",
    accent: "#c95f2f",
    ok: "#2d8f5c",
    danger: "#b63f2d",
    border: "#e2d5c4"
  },
  night: {
    bg: "#131922",
    panel: "#1b2533",
    panelSoft: "#233043",
    text: "#e8eef7",
    muted: "#9fb1c7",
    accent: "#f28d47",
    ok: "#4bbc84",
    danger: "#d86d61",
    border: "#314257"
  }
};
