import { Html, Head, Main, NextScript } from "next/document";
import Script from "next/script";

export default function Document() {
  return (
    <Html className="docs">
      <Head />
      <Script
        src="https://analytics.514.dev/script.js"
        data-host="https://moosefood.514.dev"
        data-event="PageViewEvent/0.5"
      />
      <body>
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}
