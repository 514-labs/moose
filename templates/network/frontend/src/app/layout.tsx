import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Network Fiber Panel Dashboard",
  description: "Monitoring fiber panel utilization and transport schedules",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${inter.className} bg-gray-50`}>
        <nav className="bg-blue-600 text-white p-4 shadow-md">
          <div className="container mx-auto flex justify-between items-center">
            <h1 className="text-xl font-bold">Network Dashboard</h1>
            <div className="flex gap-4">
              <a href="/" className="hover:underline">
                Fiber Panels
              </a>
              <a href="/rack-utilization" className="hover:underline">
                Rack Utilization
              </a>
              <a href="/transport" className="hover:underline">
                Transport Schedules
              </a>
            </div>
          </div>
        </nav>
        <main className="container mx-auto py-4">{children}</main>
        <footer className="bg-gray-100 border-t p-4 text-center text-gray-500 text-sm">
          <p>Network Monitoring Dashboard - © 2023</p>
        </footer>
      </body>
    </html>
  );
}
