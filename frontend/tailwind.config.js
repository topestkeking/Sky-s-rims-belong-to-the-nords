/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        darkBg: '#0b0f19',
        panelBg: '#111827',
        borderBg: '#1f293d',
        accentPrimary: '#6366f1',
        accentGreen: '#10b981',
      }
    },
  },
  plugins: [],
}
