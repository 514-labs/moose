import type { Metadata } from "next";
import "@514labs/design-system-base/globals.css";
import localFont from "next/font/local";
import { ThemeProvider } from "@514labs/design-system-components/components";
import { cn } from "@514labs/design-system-components/utils";
import { Nav } from "./nav";
import Script from "next/script";
import { GoogleTagManager } from "@next/third-parties/google";

const monoFont = localFont({
  src: "./ABCMonumentGroteskMonoVariable.woff2",
  display: "swap",
  variable: "--font-grotesk-mono",
});

const sansFont = localFont({
  src: "./ABCMonumentGroteskVariable.woff2",
  display: "swap",
  variable: "--font-grotesk",
});

export const metadata: Metadata = {
  title: "Moose—A data engineering framework for all developers.",
  description: "A data engineering framework for all developers.",
  metadataBase: new URL("https://www.moosejs.com"),
};

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  // const ip_obj = await sendServerEvent("layout-render", {
  //   layout: "root-layout",
  // });

  return (
    <html lang="en" suppressHydrationWarning>
      <Script
        src="https://analytics.514.dev/script.js"
        data-host="https://moosefood.514.dev"
        data-event="PageViewEvent/0.5"
      />
      <GoogleTagManager gtmId="GTM-MV3LQHHX" />
      <body
        className={cn(
          "min-h-screen bg-background font-sans antialiased",
          monoFont.variable,
          sansFont.variable,
        )}
        suppressHydrationWarning
      >
        <ThemeProvider
          attribute="class"
          defaultTheme="dark"
          enableSystem
          disableTransitionOnChange
        >
          <Nav />
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}
