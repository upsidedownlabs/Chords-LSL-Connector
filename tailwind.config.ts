import type { Config } from "tailwindcss";

const config = {
  darkMode: ["class"],
  content: [
    "./pages/**/*.{ts,tsx}",
    "./components/**/*.{ts,tsx}",
    "./app/**/*.{ts,tsx}",
    "./src/**/*.{ts,tsx}",
  ],
  prefix: "",
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      screens: {
        '2xl': '1400px',
      },
      fontFamily: {
        rancho: ['Rancho', 'sans-serif'],
      },
      textShadow: {
        'shadow-multiple': '1px 1px 2px rgba(0, 0, 0, 0.3), 0 0 5px rgba(0, 0, 0, 0.1)',
      },
      colors: {
        plot: "#21bbdc",
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      borderRadius: {
        lg: "var(--radius, 0.5rem)", // Falls back to 0.5rem if --radius isn't defined
        md: "calc(var(--radius, 0.5rem) - 2px)",
        sm: "calc(var(--radius, 0.5rem) - 4px)",
        DEFAULT: "0.5rem", // Explicit default
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
        fadeIn: {
          '0%': { opacity: '0', transform: 'translateY(-20px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        rotateShadow: {
          '0%': { boxShadow: '8px 8px 16px #bebebe, -8px -8px 16px #ffffff' },
          '25%': { boxShadow: '8px -8px 16px #bebebe, -8px 8px 16px #ffffff' },
          '50%': { boxShadow: '-8px -8px 16px #bebebe, 8px 8px 16px #ffffff' },
          '75%': { boxShadow: '-8px 8px 16px #bebebe, 8px -8px 16px #ffffff' },
          '100%': { boxShadow: '8px 8px 16px #bebebe, -8px -8px 16px #ffffff' },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        fadeIn: "fadeIn 0.6s ease-in-out forwards",
        rotateShadow: "rotateShadow 1.5s linear infinite",
      },
      boxShadow: {
        active: 'inset 4px 4px 8px #bebebe, inset -4px -4px 8px #ffffff',
      },
      scale: {
        95: "0.95",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
} satisfies Config;

export default config;
