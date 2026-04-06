/** System UI stack (chrome, labels, titles); no webfonts. */
const fontSystem = [
  'system-ui',
  '-apple-system',
  'BlinkMacSystemFont',
  '"Segoe UI"',
  'Roboto',
  '"Helvetica Neue"',
  'Arial',
  'sans-serif',
];

/**
 * Right-pane reading: Georgia first so Latin digits stay lining and consistent;
 * ui-serif after (e.g. New York) can use old-style figures that look odd in prose.
 */
const fontReadSerif = [
  'Georgia',
  'ui-serif',
  'Cambria',
  '"Times New Roman"',
  'Times',
  '"Liberation Serif"',
  '"Songti SC"',
  '"STSong"',
  'serif',
];

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      fontFamily: {
        sans: fontSystem,
        /** Wordmark only */
        'inkly-logo': ['Georgia', 'serif'],
        'inkly-read': fontReadSerif,
        /** Same as sans — headings / UI next to serif body */
        'inkly-read-ui': fontSystem,
      },
      colors: {
        inkly: {
          shell: '#e4dfd4',
          sidebar: '#d8d3c8',
          'sidebar-deep': '#cec8bc',
          toolbar: '#e8e3d9',
          paper: '#faf8f3',
          'paper-warm': '#f5f2ea',
          ink: '#2a2620',
          'ink-soft': '#3d3830',
          muted: '#6b655a',
          faint: '#756e62',
          line: '#c4bdb0',
          border: '#c9c2b5',
          'border-soft': '#ddd6ca',
          link: '#5c4d3d',
          'link-hover': '#3d3228',
          accent: '#4a5c4e',
          'accent-hover': '#3d4d42',
        },
      },
    },
  },
  plugins: [],
};
