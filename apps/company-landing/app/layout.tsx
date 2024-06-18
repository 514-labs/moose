import type { Metadata } from "next";
import "@514labs/design-system-base/globals.css";
import localFont from "next/font/local";
import { cn } from "@514labs/design-system-components/utils";
import { ThemeProvider } from "@514labs/design-system-components/components";
import { Nav } from "@514labs/design-system-components/trackable-components";
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
  title: "514—A data platform for all devs",
  description:
    "We build the frameworks, workflows and infrastructure that make data accessible to all developers.",
  metadataBase: new URL("https://www.fiveonefour.com"),
};

const default_navigation = [
  { name: "blog", href: "/blog" },
  { name: "about", href: "/about" },
  {
    name: "contact",
    href: "https://fiveonefour.typeform.com/signup",
  },
  { name: "get moose", href: "https://www.moosejs.com", emphasized: true },
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
          defaultTheme="dark"
          enableSystem
          disableTransitionOnChange
        >
          <Nav property="fiveonefour" navigation={default_navigation} />
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}
