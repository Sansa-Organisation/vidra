/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'vd-bg': '#0d1117',
        'vd-panel': '#161b22',
        'vd-border': '#30363d',
        'vd-text': '#e6edf3',
        'vd-dim': '#8b949e',
        'vd-accent': '#58a6ff',
        'vd-accent-hover': '#1f6feb',
      }
    },
  },
  plugins: [],
}
