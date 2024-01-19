const path = require("path");

module.exports = {
  output: "standalone",
  experimental: {
    // this includes files from the monorepo base two directories up
    outputFileTracingRoot: path.join(__dirname, "../../"),
  },
  reactStrictMode: true,
};
