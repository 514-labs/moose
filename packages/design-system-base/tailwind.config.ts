import type { Config } from "tailwindcss";

const config = {
  darkMode: ["class"],
  content: [
    "./pages/**/*.{ts,tsx, md, mdx}",
    "./components/**/*.{ts,tsx, md, mdx}",
    "./app/**/*.{ts,tsx, md, mdx}",
    "./src/**/*.{ts,tsx, md, mdx}",
    "./mdx-components.tsx",
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
        // Moose Accents
        "moose-green": {
          DEFAULT: "hsl(var(--accent-moo-green))",
          foreground: "hsl(var(--accent-moo-green-foreground))",
        },
        "moose-indigo": {
          DEFAULT: "hsl(var(--accent-moo-indigo))",
          foreground: "hsl(var(--accent-moo-indigo-foreground))",
        },
        "moose-purple": {
          DEFAULT: "hsl(var(--accent-moo-purple))",
          foreground: "hsl(var(--accent-moo-purple-foreground))",
        },
        "moose-pink": {
          DEFAULT: "hsl(var(--accent-moo-pink))",
          foreground: "hsl(var(--accent-moo-pink-foreground))",
        },
        "moose-yellow": {
          DEFAULT: "hsl(var(--accent-moo-yellow))",
          foreground: "hsl(var(--accent-moo-yellow-foreground))",
        },
        // Boreal Accents
        "boreal-teal": {
          DEFAULT: "hsl(var(--accent-bor-tea))",
          foreground: "hsl(var(--accent-bor-tea-foreground))",
        },
        "boreal-green": {
          DEFAULT: "hsl(var(--accent-bor-green))",
          foreground: "hsl(var(--accent-bor-green-foreground))",
        },
        "boreal-yellow": {
          DEFAULT: "hsl(var(--accent-bor-yel))",
          foreground: "hsl(var(--accent-bor-yel-foreground))",
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
