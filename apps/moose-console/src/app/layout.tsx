import "styles/globals.css";
import { ThemeProvider } from "../components/theme-provider";
import { cn } from "../lib/utils";

import localFont from "next/font/local";

import { Analytics } from "@vercel/analytics/react";
import { ReactNode } from "react";
import { ThemeToggle } from "../components/ui/theme-toggle";
import { TopNavMenu } from "components/top-nav-menu";

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
          <div className=" h-screen w-full flex flex-col">
            <nav className="flex flex-row w-screen px-4">
              <header className="flex text-lg">
                <a className="py-4" href={"/"}>
                  <span className="py-4">
                    moosejs{" "}
                    <span className="text-muted-foreground">console</span>
                  </span>
                </a>
                <span className="flex-grow" />
              </header>
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
            {/* <nav className="flex flex-col px-4">
              <div className="flex-grow overflow-y-auto">
                <header className="flex text-lg">
                  <span className="py-4">moosejs <span className="text-muted-foreground">console</span></span>
                  <span className="flex-grow" />
                </header>
                <LeftNav />
              </div>
              <div className="py-3 flex flex-row align-middle justify-center ">
                <ThemeToggle /> <span className="flex-grow" /> 
              </div>
              
            </nav> */}
            <section className="flex flex-grow overflow-hidden">
              {children}
            </section>
          </div>
        </ThemeProvider>
        <Analytics />
      </body>
    </html>
  );
}
