import "styles/globals.css";
import { ThemeProvider } from "../components/theme-provider";
import { cn } from "../lib/utils";

import localFont from "next/font/local";

import { Analytics } from "@vercel/analytics/react";
import { ReactNode } from "react";
import { ThemeToggle } from "../components/ui/theme-toggle";
import { LeftNav } from "../components/left-nav";

// Font files can be colocated inside of `app`
const monoFont = localFont({
  src: "./ABCMonumentGroteskMonoVariable.woff2",
  display: "swap",
  variable: "--grotesk-mono",
});

const sansFont = localFont({
  src: "./ABCMonumentGroteskVariable.woff2",
  display: "swap",
  variable: "--grotesk-sans",
});

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}): ReactNode {
  return (
    <html
      lang="en"
      suppressHydrationWarning
      className={cn(sansFont.variable, monoFont.variable)}
    >
      <body className={cn("min-h-screen font-sans antialiased")}>
        <ThemeProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange
        >
          <header className="flex text-lg px-3">
            <span className="py-4">moosejs</span>
            <span className="flex-grow" />
            <span className="py-3">
              <ThemeToggle />
            </span>
          </header>
          <div className="flex">
            <nav>
              <LeftNav />
            </nav>
            <section>{children}</section>
          </div>
        </ThemeProvider>
        <Analytics />
      </body>
    </html>
  );
}
