module.exports = {
  extends: ["next", "turbo", "prettier", "eslint:recommended", 'plugin:@typescript-eslint/recommended'],
  parser: "@typescript-eslint/parser",
  plugins: ["@typescript-eslint"],
  rules: {
    "@next/next/no-html-link-for-pages": "off",
    // Can mark a variable or an arg as unused by prefixing with _
    "@typescript-eslint/no-unused-vars": ["error", {
      "argsIgnorePattern": "^_",
      "varsIgnorePattern": "^_",
      "caughtErrorsIgnorePattern": "^_"
    }],
    // Temporarily (?) disabled
    "@typescript-eslint/no-explicit-any": "off",
  },
  parserOptions: {
    babelOptions: {
      presets: [require.resolve("next/babel")],
    },
  },
};
