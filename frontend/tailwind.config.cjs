/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        /* Transitional serif + OS CJK fallbacks (no webfont download). */
        "inkly-read": [
          "Charter",
          "Bitstream Charter",
          "Iowan Old Style",
          "Palatino Linotype",
          "Book Antiqua",
          "Palatino",
          "Cambria",
          "Sitka Text",
          "Georgia",
          "Hiragino Sans GB",
          "PingFang SC",
          "Microsoft YaHei",
          '"Source Han Sans SC"',
          '"Noto Sans SC"',
          '"Noto Sans CJK SC"',
          "serif",
        ],
        /* Title / chrome contrast against serif body */
        "inkly-read-ui": [
          "system-ui",
          "-apple-system",
          "BlinkMacSystemFont",
          '"Segoe UI"',
          '"PingFang SC"',
          '"Hiragino Sans GB"',
          '"Source Han Sans SC"',
          '"Noto Sans SC"',
          "sans-serif",
        ],
      },
      colors: {
        inkly: {
          shell: "#e4dfd4",
          sidebar: "#d8d3c8",
          "sidebar-deep": "#cec8bc",
          toolbar: "#e8e3d9",
          paper: "#faf8f3",
          "paper-warm": "#f5f2ea",
          ink: "#2a2620",
          "ink-soft": "#3d3830",
          muted: "#6b655a",
          faint: "#8a8478",
          line: "#c4bdb0",
          border: "#c9c2b5",
          "border-soft": "#ddd6ca",
          link: "#5c4d3d",
          "link-hover": "#3d3228",
          accent: "#4a5c4e",
          "accent-hover": "#3d4d42",
        },
      },
    },
  },
  plugins: [],
};

