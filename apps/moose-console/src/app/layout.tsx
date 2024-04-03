import "styles/globals.css";
import { ThemeProvider } from "../components/theme-provider";
import { cn } from "../lib/utils";

import localFont from "next/font/local";

import { ReactNode } from "react";
import { ThemeToggle } from "../components/ui/theme-toggle";
import { TopNavMenu } from "components/top-nav-menu";
import { VersionProvider } from "version-context";
import { CURRENT_VERSION } from "./types";
import VersionSelect from "components/version-select";

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
      <script
        data-host="https://moosefood.514.dev"
        src="https://analytics.514.dev/script.js"
      />
      <body className={cn("min-h-screen font-sans antialiased")}>
        <ThemeProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange
        >
          <VersionProvider version={CURRENT_VERSION}>
            <div className=" h-screen w-full flex flex-col">
              <nav className="flex flex-row w-screen px-4">
                <header className="flex text-lg">
                  <a className="py-4 content-center" href={"/"}>
                    <span className="py-4">
                      moosejs{" "}
                      <span className="text-muted-foreground">console</span>
                    </span>
                  </a>
                  <span className="flex-grow" />
                </header>
                <VersionSelect />
                <span className="flex-grow" />
                <div className="py-3">
                  <TopNavMenu />
                </div>
                <div className="ml-3">
                  <div className="py-3 flex flex-row align-middle justify-center ">
                    <ThemeToggle /> <span className="flex-grow" />
                  </div>
                </div>
              </nav>
              <section className="flex flex-grow overflow-hidden">
                {children}
              </section>
            </div>
            <footer className="text-center py-3 text-muted-foreground">
              {process.env.NEXT_PUBLIC_RELEASE_VERSION || "dev"}
            </footer>
          </VersionProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
