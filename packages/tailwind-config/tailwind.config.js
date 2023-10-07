const colors = require("tailwindcss/colors");

const baseColors = {
  black: {
    500: "#000000",
    250: "#1A1A1A",
  },
  white: {
    500: "#FFFFFF",
    250: "#848484",
  },
  chartreuse: {
    500: "#BDFF00",
    250: "#638500",
  },
  red: {
    500: "#FF0000",
    250: "#8D0000",
  }
}

module.exports = {
  content: [
    // app content
    `src/**/*.{js,ts,jsx,tsx}`,
    // include packages if not transpiling
    // "../../packages/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        typography: {
          primary: baseColors.white[500],
          secondary: baseColors.chartreuse[500],
        },
        action: {
          primary: baseColors.chartreuse[500],
        },
        base: baseColors
      },
      fontFamily: {
        sans: ['var(--font-grotesk)'],
        mono: ['var(--font-grotesk-mono)'],
      },
    },
  },
  plugins: [],
};
