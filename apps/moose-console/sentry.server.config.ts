import * as Sentry from "@sentry/nextjs";
const isProduction = process.env.NODE_ENV === "production";

Sentry.init({
  enabled: isProduction,
  dsn: "https://420ad9a138f36701dadb42fd8640357f@o4505851966128128.ingest.us.sentry.io/4506458108919808",

  // Set tracesSampleRate to 1.0 to capture 100%
  // of transactions for performance monitoring.
  // We recommend adjusting this value in production
  tracesSampleRate: 1.0,

  // ...

  // Note: if you want to override the automatic release value, do not set a
  // `release` value here - use the environment variable `SENTRY_RELEASE`, so
  // that it will also get attached to your source maps
});
