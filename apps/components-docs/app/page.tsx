import Link from "next/link";

function Hero() {
  return (
    <div className="flex flex-col text-center items-center container py-16 max-w-2xl gap-4">
      <h1 className="text-5xl">
        Bullet-proof, customizable, connectors... as actual code
      </h1>
      <h2 className="text-lg text-muted-foreground">
        A starter kit for building, testing and sharing connectors that can be
        customomized and embedded in apps.
      </h2>
    </div>
  );
}

export default function Home() {
  return (
    <div className="font-sans items-center justify-items-center min-h-screen space-y-4">
      <main className="flex flex-col items-center sm:items-start">
        <Hero />
      </main>
      <footer className="row-start-3 flex flex-wrap items-center justify-center">
        <span>
          Inspired by
          <Link
            className="pl-1"
            href="https://ui.shadcn.com"
            target="_blank"
            rel="noopener noreferrer"
          >
            shadcn/ui
          </Link>
        </span>
        <span>
          , Created with ❤️ by the folks at
          <Link
            className="pl-1"
            href="https://www.fiveonefour.com"
            target="_blank"
            rel="noopener noreferrer"
          >
            514 Labs
          </Link>
        </span>
      </footer>
    </div>
  );
}
