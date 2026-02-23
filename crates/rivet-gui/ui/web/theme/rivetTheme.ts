import { createTheme } from "@mui/material/styles";

export type ThemeMode = "day" | "night";

export function createRivetTheme(mode: ThemeMode) {
  const isNight = mode === "night";

  return createTheme({
    palette: {
      mode: isNight ? "dark" : "light",
      primary: {
        main: isNight ? "#f28d47" : "#c95f2f"
      },
      success: {
        main: isNight ? "#4bbc84" : "#2d8f5c"
      },
      error: {
        main: isNight ? "#d86d61" : "#b63f2d"
      },
      background: {
        default: isNight ? "#131922" : "#f4efe7",
        paper: isNight ? "#1b2533" : "#fffdf7"
      },
      text: {
        primary: isNight ? "#e8eef7" : "#241d17",
        secondary: isNight ? "#9fb1c7" : "#6c5d4f"
      },
      divider: isNight ? "#314257" : "#e2d5c4"
    },
    shape: {
      borderRadius: 14
    },
    components: {
      MuiPaper: {
        styleOverrides: {
          root: {
            border: "1px solid",
            borderColor: isNight ? "#314257" : "#e2d5c4",
            boxShadow: isNight ? "0 10px 28px rgba(0, 0, 0, 0.35)" : "0 8px 22px rgba(29, 17, 8, 0.05)"
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
