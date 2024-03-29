import type { Metadata } from "next";
import "design-system/globals.css";
import localFont from "next/font/local";
import { Nav } from "design-system/components";
import { ThemeProvider } from "design-system/components";
import { cn } from "design-system/utils";

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
  title: "Create Next App",
  description: "Generated by create next app",
};

const default_navigation = [
  { name: "docs", href: "https://docs.moosejs.com" },
  { name: "templates", href: "/templates" },
  { name: "blog", href: "https://blog.fiveonefour.com/" },
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
