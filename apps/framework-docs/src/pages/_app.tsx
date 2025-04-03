import "@/styles/globals.css";
import RootLayout from "@/components/layouts";
import { useEffect } from "react";
import { useRouter } from "next/router";
import Router from "next/router";
import posthog from "posthog-js";
import Script from "next/script";
import type { AppProps } from "next/app";

export default function App({ Component, pageProps }: AppProps) {
  const router = useRouter();

  useEffect(() => {
    // Wait until Next.js router is ready to ensure query params are populated.
    if (!router.isReady) return;

    // Extract cross-site tracking params from Next.js router query.
    // These are added by TrackLink as 'ph_distinct_id' and 'ph_session_id'.
    const { ph_distinct_id, ph_session_id } = router.query;
    let bootstrapData = {};

    if (ph_distinct_id || ph_session_id) {
      bootstrapData = {
        ...(ph_distinct_id && { distinct_id: ph_distinct_id }),
        ...(ph_session_id && { session_id: ph_session_id }),
      };
    }

    if (process.env.NEXT_PUBLIC_POSTHOG_KEY) {
      posthog.init(process.env.NEXT_PUBLIC_POSTHOG_KEY, {
        api_host:
          process.env.NEXT_PUBLIC_POSTHOG_HOST || "https://us.i.posthog.com",
        ui_host: "https://us.posthog.com",
        bootstrap: bootstrapData, // Bootstrap with IDs from the URL if provided.
        // Enable debug mode in development
        loaded: (posthogInstance) => {
          if (process.env.NODE_ENV === "development") posthogInstance.debug();
        },
      });
    }

    // Track page views
    const handleRouteChange = () => {
      if (process.env.NEXT_PUBLIC_POSTHOG_KEY) {
        const url = window.location.origin + router.asPath;
        posthog?.capture("$pageview", {
          $current_url: url,
        });
      }
    };

    // Track initial page view
    handleRouteChange();

    Router.events.on("routeChangeComplete", handleRouteChange);

    return () => {
      Router.events.off("routeChangeComplete", handleRouteChange);
    };
  }, [router.isReady, router.asPath]);

  return (
    <RootLayout>
      <Script
        id="apollo-tracking"
        strategy="beforeInteractive"
        dangerouslySetInnerHTML={{
          __html: `
              function initApollo() {
                var n = Math.random().toString(36).substring(7);
                var o = document.createElement("script");
                o.src = "https://assets.apollo.io/micro/website-tracker/tracker.iife.js?nocache=" + n;
                o.async = true;
                o.defer = true;
                o.onload = function() {
                  window.trackingFunctions.onLoad({
                    appId: "66316b76c8e6ae01afde8c2d"
                  });
                };
                document.head.appendChild(o);
              }
              initApollo();
            `,
        }}
      />
      <Component {...pageProps} />
    </RootLayout>
  );
}
