//import React, { useState } from "react";
// import { useClient } from "next/client";
import { usePathname } from "next/navigation";
import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import localFont from "next/font/local";
import { NavTabs } from "./nav-tabs";
import { cn } from "design-system/utils";
import { Nav } from "design-system/trackable-components";
import Script from "next/script";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Product Analytics",
  description: "Internal product analytics for 514 instrumented with moose",
};

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

const default_navigation = [
  { name: "blog", href: "/blog" },
  { name: "about", href: "/about" },
  {
    name: "get in touch",
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
        {/* <ThemeProvider
          attribute="class"
          defaultTheme="dark"
          enableSystem
          disableTransitionOnChange
        > */}
        <Nav
          property="moosejs"
          className="p-40"
          navigation={default_navigation}
        />
        <div className="text-6xl m-6 py-6">Product Analytics</div>
        <NavTabs />
        {children}
        {/* </ThemeProvider> */}
      </body>
    </html>
  );
}
