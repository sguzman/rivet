import { useEffect, useMemo } from "react";

import CssBaseline from "@mui/material/CssBaseline";
import GlobalStyles from "@mui/material/GlobalStyles";
import { ThemeProvider } from "@mui/material/styles";

import { AppShell } from "./app/AppShell";
import { useAppStore } from "./store/useAppStore";
import { createRivetTheme } from "./theme/rivetTheme";

export default function App() {
  const themeMode = useAppStore((state) => state.themeMode);
  const themeFollowSystem = useAppStore((state) => state.themeFollowSystem);
  const systemThemeMode = useAppStore((state) => state.systemThemeMode);
  const setSystemThemeMode = useAppStore((state) => state.setSystemThemeMode);
  const resolvedThemeMode = themeFollowSystem ? systemThemeMode : themeMode;
  const theme = useMemo(() => createRivetTheme(resolvedThemeMode), [resolvedThemeMode]);

  useEffect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
      return;
    }
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      setSystemThemeMode(media.matches ? "night" : "day");
    };
    apply();
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [setSystemThemeMode]);

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <GlobalStyles
        styles={{
          body: {
            margin: 0,
            minHeight: "100vh",
            background: theme.palette.background.default,
            color: theme.palette.text.primary
          },
          "#app": {
            minHeight: "100vh"
          }
        }}
      />
      <div className={resolvedThemeMode === "night" ? "dark" : ""}>
        <AppShell />
      </div>
    </ThemeProvider>
  );
}
