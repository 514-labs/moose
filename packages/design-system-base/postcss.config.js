module.exports = {
  plugins: [
    "tailwindcss",
    [
      "autoprefixer",
      {
        // Add any specific autoprefixer options here if needed
        flexbox: true,
        grid: true,
      },
    ],
  ],
};
