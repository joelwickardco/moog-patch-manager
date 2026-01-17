/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,svelte}"],
  theme: {
    extend: {
      colors: {
        // Primary brand colors
        primary: "#FF6B35",
        secondary: "#004E89",
        accent: "#F7B801",
        // Semantic
        favorite: "#FFD700",
        // Category defaults
        category: {
          bass: "#8B4513",
          lead: "#FF6347",
          pad: "#9370DB",
          fx: "#20B2AA",
        },
        // UI colors (dark mode first)
        background: "#1A1A1A",
        surface: "#2D2D2D",
        border: "#404040",
        "text-primary": "#FFFFFF",
        "text-secondary": "#B0B0B0",
      },
    },
  },
  plugins: [],
};
