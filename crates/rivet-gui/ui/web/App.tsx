import { useMemo } from "react";

import CssBaseline from "@mui/material/CssBaseline";
import GlobalStyles from "@mui/material/GlobalStyles";
import { ThemeProvider } from "@mui/material/styles";

import { AppShell } from "./app/AppShell";
import { useAppStore } from "./store/useAppStore";
import { createRivetTheme } from "./theme/rivetTheme";

export default function App() {
  const themeMode = useAppStore((state) => state.themeMode);
  const theme = useMemo(() => createRivetTheme(themeMode), [themeMode]);

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
      <div className={themeMode === "night" ? "dark" : ""}>
        <AppShell />
      </div>
    </ThemeProvider>
  );
}
