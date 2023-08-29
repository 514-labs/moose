import "../styles/globals.css";
// include styles from the ui package
import "ui/styles.css";
import localFont from 'next/font/local'

import { Analytics } from '@vercel/analytics/react';

// Font files can be colocated inside of `app`
const monoFont = localFont({
  src: './ABCMonumentGroteskMonoVariable.woff2',
  display: 'swap',
  variable: '--font-grotesk-mono'
});

const sansFont = localFont({
  src: './ABCMonumentGroteskVariable.woff2',
  display: 'swap',
  variable: '--font-grotesk'
});


export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en"  className={"bg-black h-full " + `${monoFont.variable} ${sansFont.variable}`}>
      <body className="h-full font-sans font-regular">
        {children}
        <Analytics />
      </body>
    </html>
  );
}
