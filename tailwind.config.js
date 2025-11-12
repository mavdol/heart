/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      keyframes: {
        "typing-bounce": {
          "0%, 80%, 100%": {
            transform: "scale(0)",
            opacity: "0.5",
          },
          "40%": {
            transform: "scale(1)",
            opacity: "1",
          },
        },
        "slide-up": {
          from: {
            opacity: "0",
            transform: "translateY(var(--slide-distance, 20px))",
          },
          to: {
            opacity: "1",
            transform: "translateY(0)",
          },
        },
        "slide-down": {
          from: {
            opacity: "0",
            transform: "translateY(calc(var(--slide-distance, 20px) * -1))",
          },
          to: {
            opacity: "1",
            transform: "translateY(0)",
          },
        },
        "slide-left": {
          from: {
            opacity: "0",
            transform: "translateX(var(--slide-distance, 20px))",
          },
          to: {
            opacity: "1",
            transform: "translateX(0)",
          },
        },
        "slide-right": {
          from: {
            opacity: "0",
            transform: "translateX(calc(var(--slide-distance, 20px) * -1))",
          },
          to: {
            opacity: "1",
            transform: "translateX(0)",
          },
        },
        "fade-in": {
          from: {
            opacity: "0",
          },
          to: {
            opacity: "1",
          },
        },
      },
      animation: {
        "typing-bounce": "typing-bounce 1.4s ease-in-out infinite",
        "slide-up": "slide-up 0.6s ease-out",
        "slide-down": "slide-down 0.6s ease-out",
        "slide-left": "slide-left 0.6s ease-out",
        "slide-right": "slide-right 0.6s ease-out",
        "fade-in": "fade-in 0.6s ease-out",
      },
      animationDuration: {
        fast: "0.3s",
        normal: "0.6s",
        slow: "1s",
      },
      animationDelay: {
        100: "0.1s",
        200: "0.2s",
        300: "0.3s",
        500: "0.5s",
        700: "0.7s",
        1000: "1s",
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      colors: {
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        chart: {
          1: "hsl(var(--chart-1))",
          2: "hsl(var(--chart-2))",
          3: "hsl(var(--chart-3))",
          4: "hsl(var(--chart-4))",
          5: "hsl(var(--chart-5))",
        },
      },
    },
  },
  plugins: [
    require("tailwindcss-animate"),
    function ({ addUtilities, theme }) {
      addUtilities({
        ".animate-distance-sm": {
          "--slide-distance": "10px",
        },
        ".animate-distance-md": {
          "--slide-distance": "20px",
        },
        ".animate-distance-lg": {
          "--slide-distance": "40px",
        },
        ".animate-distance-xl": {
          "--slide-distance": "60px",
        },
        ".animate-bounce-custom": {
          "animation-timing-function": "cubic-bezier(0.68, -0.55, 0.265, 1.55)",
        },
      });
    },
  ],
};
