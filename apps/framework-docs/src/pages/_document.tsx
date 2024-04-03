import { Html, Head, Main, NextScript } from "next/document";

export default function Document() {
  return (
    <Html className="docs">
      <Head />
      <script
        data-host="https://moosefood.514.dev"
        src="https://analytics.514.dev/script.js"
      />
      <body>
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}
