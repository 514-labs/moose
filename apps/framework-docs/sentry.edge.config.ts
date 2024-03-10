import * as Sentry from "@sentry/nextjs";

Sentry.init({
  dsn: "https://06b5279a67c3c370fba51bb4a8bae624@o4505851966128128.ingest.us.sentry.io/4506504179482624",

  // Set tracesSampleRate to 1.0 to capture 100%
  // of transactions for performance monitoring.
  // We recommend adjusting this value in production
  tracesSampleRate: 1.0,

  // ...

  // Note: if you want to override the automatic release value, do not set a
  // `release` value here - use the environment variable `SENTRY_RELEASE`, so
  // that it will also get attached to your source maps
});
