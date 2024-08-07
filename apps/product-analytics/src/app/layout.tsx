import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { NavTabs } from "./nav-tabs";
import NextAuthProvider from "./next-auth-provider";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Product Analytics",
  description: "Internal product analytics for 514 instrumented with moose",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <nav className="flex flex-row w-screen px-4">
          <header className="flex text-lg">
            <a className="py-4" href={"/"}>
              <span className="py-4">moosejs</span>
            </a>
            <span className="flex-grow" />
          </header>
        </nav>
        <div className="text-6xl m-4">Product Analytics</div>
        <NavTabs />
        <NextAuthProvider>{children}</NextAuthProvider>
      </body>
    </html>
  );
}
