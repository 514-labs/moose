import nextra from "nextra";
import remarkMdxDisableExplicitJsx from "remark-mdx-disable-explicit-jsx";

// Disable Next.js telemetry
process.env.NEXT_TELEMETRY_DISABLED = "1";

const withNextra = nextra({
  theme: "nextra-theme-docs",
  themeConfig: "./theme.config.jsx",
  defaultShowCopyCode: true,
  staticImage: true,
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
  transpilePackages: [],
  reactStrictMode: true,
  images: {
    unoptimized: true,
    domains: ["docs.fiveonefour.com"],
  },
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
  telemetry: false,
};

// Make sure adding Sentry options is the last code to run before exporting
export default withNextra(nextConfig);

// Make sure adding Sentry options is the last code to run before exporting
// module.exports = withNextra(
//   withSentryConfig(nextConfig, sentryWebpackPluginOptions)
// );

// If you're using a next.config.mjs file:
// export default withSentryConfig(nextConfig, sentryWebpackPluginOptions);
