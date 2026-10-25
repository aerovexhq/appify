/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
    "../../strata/packages/ui/src/**/*.{js,ts,jsx,tsx}"
  ],
  theme: {
    extend: {
      colors: {
        bg: "var(--k-bg)",
        surface: "var(--k-surface)",
        raised: "var(--k-raised)",
        border: "var(--k-border)",
        fg: "var(--k-fg)",
        muted: "var(--k-muted)",
        accent: "var(--k-accent)",
        "accent-fg": "var(--k-accent-fg)",
        danger: "var(--k-danger, #ef4444)",
        aerovex: {
          canvas: "var(--bg-canvas)",
          "canvas-subtle": "var(--bg-canvas-subtle)",
          surface: "var(--bg-surface)",
          card: "var(--bg-card)",
          "card-hover": "var(--bg-card-hover)",
          "card-active": "var(--bg-card-active)",
          hud: "var(--bg-hud)",
          border: "var(--border-subtle)",
          "border-hairline": "var(--border-hairline)",
          accent: "var(--text-accent)",
          "accent-hover": "var(--text-accent-hover)",
          text: "var(--text-primary)",
          "text-secondary": "var(--text-secondary)",
          "text-muted": "var(--text-muted)"
        }
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
