import type { Config } from "tailwindcss";

const config = {
  darkMode: ["class"],
  content: [
    "./pages/**/*.{ts,tsx, md, mdx}",
    "./components/**/*.{ts,tsx, md, mdx}",
    "./app/**/*.{ts,tsx, md, mdx}",
    "./src/**/*.{ts,tsx, md, mdx}",
    "./mdx-components.tsx",
    "./theme.config.jsx",
    "../../packages/design-system-base/**/*.{ts,tsx, md, mdx}",
    "../../packages/design-system-components/**/*.{ts,tsx, md, mdx}",
    "../../packages/event-capture/**/*.{ts,tsx, md, mdx}",
  ],
  // safelist: ["list-disc", "list-decimal", "list-inside", "list-item", "pl-5", "aspect-video"],
  prefix: "",
  theme: {
    container: {
      center: true,
      padding: "2rem",
    },
    extend: {
      screens: {
        "3xl": "1792px",
      },
      colors: {
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
        blue: {
          DEFAULT: "hsl(var(--accent-blue))",
        },
        teal: {
          DEFAULT: "hsl(var(--accent-teal))",
        },
        indigo: {
          DEFAULT: "hsl(var(--accent-indigo))",
          trans: "hsl(var(--accent-indigo-trans))/20",
        },
        purple: {
          DEFAULT: "hsl(var(--accent-purple))",
        },
        pink: {
          DEFAULT: "hsl(var(--accent-pink))",
        },
        yellow: {
          DEFAULT: "hsl(var(--accent-yellow))",
        },
        green: {
          DEFAULT: "hsl(var(--accent-green))",
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
      backgroundImage: {
        gradient: "var(--moose-gradient)",
        gradientDark: "var(--moose-gradient-dark)",
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
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
      },
      animation: {
        "accordion-down": "accordion-down 0.5s ease-out",
        "accordion-up": "accordion-up 0.5s ease-out",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
} satisfies Config;

export default config;
