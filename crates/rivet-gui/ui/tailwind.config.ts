import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: "class",
  content: ["./index.html", "./web/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        rivet: {
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
        rivetNight: {
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
      },
      boxShadow: {
        panel: "0 10px 28px rgba(0, 0, 0, 0.18)"
      }
    }
  },
  plugins: []
};

export default config;
