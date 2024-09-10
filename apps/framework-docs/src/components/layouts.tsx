import localFont from "next/font/local";
// Font files can be colocated inside of `app`
const monoFont = localFont({
  src: "../ABCMonumentGroteskMonoVariable.woff2",
  display: "swap",
  variable: "--font-grotesk-mono",
});

const sansFont = localFont({
  src: "../ABCMonumentGroteskVariable.woff2",
  display: "swap",
  variable: "--font-grotesk",
});

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <main
      lang="en"
      className={"font-sans" + ` ${monoFont.variable} ${sansFont.variable}`}
    >
      {children}
    </main>
  );
}
