import type { Metadata } from "next";
import "design-system/globals.css";
import localFont from "next/font/local";
import { ThemeProvider } from "design-system/components";
import { cn } from "design-system/utils";
import { Nav } from "design-system/trackable-components";
import Script from "next/script";

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
  title: "MooseJS—A data engineering framework for all developers.",
  description: "A data engineering framework for all developers.",
  openGraph: {
    images: "/images/open-graph/og_moose_4x.png",
  },
};

const default_navigation = [
  { name: "docs", href: "https://docs.moosejs.com" },
  { name: "templates", href: "/templates" },
  { name: "community", href: "/community" },
  // { name: "blog", href: "https://blog.fiveonefour.com/" },
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
    <html lang="en" suppressHydrationWarning className="">
      <Script
        src="https://analytics.514.dev/script.js"
        data-host="https://moosefood.514.dev"
      />
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
          defaultTheme="light"
          enableSystem
          disableTransitionOnChange
        >
          <Nav property="moosejs" navigation={default_navigation} />
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}
