import * as Sentry from "@sentry/nextjs";

Sentry.init({
  dsn: "https://ad3e8eb464c321b18b78c8402e80a8e5@o4505851966128128.ingest.us.sentry.io/4505852583084032",
  // Replay may only be enabled for the client-side
  integrations: [Sentry.replayIntegration(), Sentry.feedbackIntegration({
    // Additional SDK configuration goes in here, for example:
    colorScheme: "system",
  })],

  // Set tracesSampleRate to 1.0 to capture 100%
  // of transactions for performance monitoring.
  // We recommend adjusting this value in production
  tracesSampleRate: 1.0,

  // Capture Replay for 10% of all sessions,
  // plus for 100% of sessions with an error
  replaysSessionSampleRate: 0.1,
  replaysOnErrorSampleRate: 1.0,

  // ...

  // Note: if you want to override the automatic release value, do not set a
  // `release` value here - use the environment variable `SENTRY_RELEASE`, so
  // that it will also get attached to your source maps
});
