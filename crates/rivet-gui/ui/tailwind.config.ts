import type { Config } from "tailwindcss";

import { RIVET_TOKENS } from "./web/theme/tokens";

const config: Config = {
  darkMode: "class",
  content: ["./index.html", "./web/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        rivet: {
          ...RIVET_TOKENS.day
        },
        rivetNight: {
          ...RIVET_TOKENS.night
        }
      },
      boxShadow: {
        panel: RIVET_TOKENS.shadows.nightPanel
      }
    }
  },
  plugins: []
};

export default config;
