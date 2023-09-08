import "../styles/globals.css";
// include styles from the ui package
import "ui/styles.css";
import localFont from 'next/font/local'
import Head from 'next/head';

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
      <Head>
      <script type="text/javascript">
        window.heap=window.heap||[],heap.load=function(e,t){window.heap.appid=e,window.heap.config=t=t||{};var r=document.createElement("script");r.type="text/javascript",r.async=!0,r.src="https://cdn.heapanalytics.com/js/heap-"+e+".js";var a=document.getElementsByTagName("script")[0];a.parentNode.insertBefore(r,a);for(var n=function(e){return function(){heap.push([e].concat(Array.prototype.slice.call(arguments,0)))}},p=["addEventProperties","addUserProperties","clearEventProperties","identify","resetIdentity","removeEventProperty","setEventProperties","track","unsetEventProperty"],o=0;o<p.length;o++)heap[p[o]]=n(p[o])};
        heap.load("4099753721");
      </script>
      </Head>
      <body className="h-full font-sans font-regular">
        {children}
        <Analytics />
      </body>
    </html>
  );
}
