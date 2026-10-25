/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
    "../../strata/packages/ui/src/**/*.{js,ts,jsx,tsx}"
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['"Inter Variable"', 'system-ui', 'sans-serif'],
        mono: ['"JetBrains Mono Variable"', 'ui-monospace', 'monospace'],
      },
      colors: {
        bg: "rgb(var(--k-bg-rgb) / <alpha-value>)",
        surface: "rgb(var(--k-surface-rgb) / <alpha-value>)",
        raised: "rgb(var(--k-raised-rgb) / <alpha-value>)",
        border: "rgb(var(--k-border-rgb) / <alpha-value>)",
        fg: "rgb(var(--k-fg-rgb) / <alpha-value>)",
        muted: "rgb(var(--k-muted-rgb) / <alpha-value>)",
        accent: {
          DEFAULT: "rgb(var(--k-accent-rgb) / <alpha-value>)",
          fg: "rgb(var(--k-accent-fg-rgb) / <alpha-value>)",
        },
        danger: "rgb(var(--k-danger-rgb) / <alpha-value>)",
      },
      borderRadius: {
        sm: "var(--radius-sm)",
        md: "var(--radius-md)",
        lg: "var(--radius-lg)",
        pill: "9999px"
      }
    }
  },
  plugins: []
};
