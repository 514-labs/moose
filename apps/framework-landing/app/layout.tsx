import type { Metadata } from "next";
import "@514labs/design-system-base/globals.css";
import localFont from "next/font/local";
import { ThemeProvider } from "@514labs/design-system-components/components";
import { cn } from "@514labs/design-system-components/utils";
import { Nav } from "@514labs/design-system-components/trackable-components";
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

const default_navigation = [
  { name: "docs", href: "https://docs.moosejs.com" },
  { name: "boreal", href: "https://boreal.cloud" },
  {
    name: "templates",
    href: "/templates",
    items: [
      { name: "product analytics", href: "/templates/product-analytics" },
    ],
  },
  { name: "community", href: "/community" },
  { name: "blog", href: "https://www.fiveonefour.com/blog" },
  { name: "github", href: "https://github.com/514-labs/moose" },
];

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
          <Nav
            property="Moose"
            subProperty="JS PY"
            navigation={default_navigation}
            className="bg-gradient"
          />
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}
