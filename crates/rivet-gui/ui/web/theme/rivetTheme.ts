import { createTheme } from "@mui/material/styles";

import { RIVET_TOKENS } from "./tokens";

export type ThemeMode = "day" | "night";

export function createRivetTheme(mode: ThemeMode) {
  const isNight = mode === "night";
  const palette = isNight ? RIVET_TOKENS.night : RIVET_TOKENS.day;

  return createTheme({
    palette: {
      mode: isNight ? "dark" : "light",
      primary: {
        main: palette.accent
      },
      success: {
        main: palette.ok
      },
      error: {
        main: palette.danger
      },
      background: {
        default: palette.bg,
        paper: palette.panel
      },
      text: {
        primary: palette.text,
        secondary: palette.muted
      },
      divider: palette.border
    },
    shape: {
      borderRadius: RIVET_TOKENS.radius.panel
    },
    typography: {
      fontFamily: RIVET_TOKENS.typography.bodyFontFamily
    },
    components: {
      MuiPaper: {
        styleOverrides: {
          root: {
            border: "1px solid",
            borderColor: palette.border,
            boxShadow: isNight ? RIVET_TOKENS.shadows.nightPanel : RIVET_TOKENS.shadows.dayPanel
          }
        }
      },
      MuiDialog: {
        styleOverrides: {
          paper: {
            minWidth: 520
          }
        }
      }
    }
  });
}
