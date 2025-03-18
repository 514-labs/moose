const remarkMdxDisableExplicitJsx = import("remark-mdx-disable-explicit-jsx");

const withNextra = require("nextra")({
  theme: "nextra-theme-docs",
  themeConfig: "./theme.config.jsx",

  mdxOptions: {
    remarkPlugins: [
      [
        remarkMdxDisableExplicitJsx,
        { whiteList: ["table", "thead", "tbody", "tr", "th", "td"] },
      ],
    ],
  },
});

// your existing module.exports or default export
const nextConfig = {
  transpilePackages: ["@514labs/design-system"],
  reactStrictMode: true,
  // Optional build-time configuration options
  async rewrites() {
    return [
      {
        source: "/ingest/static/:path*",
        destination: "https://us-assets.i.posthog.com/static/:path*",
      },
      {
        source: "/ingest/:path*",
        destination: "https://us.i.posthog.com/:path*",
      },
      {
        source: "/ingest/decide",
        destination: "https://us.i.posthog.com/decide",
      },
    ];
  },
  skipTrailingSlashRedirect: true,
};

// Make sure adding Sentry options is the last code to run before exporting
module.exports = withNextra(nextConfig);

// Make sure adding Sentry options is the last code to run before exporting
// module.exports = withNextra(
//   withSentryConfig(nextConfig, sentryWebpackPluginOptions)
// );

// If you're using a next.config.mjs file:
// export default withSentryConfig(nextConfig, sentryWebpackPluginOptions);
