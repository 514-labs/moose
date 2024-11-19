import * as Sentry from "@sentry/node";
import { nodeProfilingIntegration } from "@sentry/profiling-node";
import { cliLog } from "./commons";

const SENTRY_DSN = process.env["SENTRY_DSN"];
const MOOSE_ENVIRONMENT = process.env["MOOSE_ENVIRONMENT"];

if (SENTRY_DSN) {
  // Ensure to call this before importing any other modules!
  Sentry.init({
    dsn: SENTRY_DSN,
    environment: MOOSE_ENVIRONMENT,
    integrations: [
      // Add our Profiling integration
      nodeProfilingIntegration(),
    ],

    // Add Tracing by setting tracesSampleRate
    // We recommend adjusting this value in production
    tracesSampleRate: 1.0,

    // Set sampling rate for profiling
    // This is relative to tracesSampleRate
    profilesSampleRate: 1.0,
  });
}
