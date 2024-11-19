import * as Sentry from "@sentry/node";
import { nodeProfilingIntegration } from "@sentry/profiling-node";

const SENTRY_DSN = process.env["SENTRY_DSN"];
const SENTRY_TRACE_SAMPLE_RATE = process.env["SENTRY_TRACE_SAMPLE_RATE"];
const SENTRY_PROFILE_SAMPLE_RATE = process.env["SENTRY_PROFILE_SAMPLE_RATE"];
const MOOSE_ENVIRONMENT = process.env["MOOSE_ENVIRONMENT"];

const sentryTraceSampleRate =
  (SENTRY_TRACE_SAMPLE_RATE && parseFloat(SENTRY_TRACE_SAMPLE_RATE)) || 1.0;
const sentryProfileSampleRate =
  (SENTRY_PROFILE_SAMPLE_RATE && parseFloat(SENTRY_PROFILE_SAMPLE_RATE)) || 0.1;

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
    tracesSampleRate: sentryTraceSampleRate,

    // Set sampling rate for profiling
    // This is relative to tracesSampleRate
    profilesSampleRate: sentryProfileSampleRate,
  });
}
