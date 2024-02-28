const path = require("path");

module.exports = {
  webpack: (config) => {
    config.externals.push({
      canvas: "commonjs canvas"
    })
    return config
  },
  output: "standalone",
  experimental: {
    // this includes files from the monorepo base two directories up
    outputFileTracingRoot: path.join(__dirname, "../../"),
  },
  reactStrictMode: true,
};
