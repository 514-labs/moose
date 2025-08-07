import Link from "next/link";
import { ThemeToggle } from "./theme-toggle";
import { Button } from "./ui/button";

export function SiteHeader() {
  return (
    <header className="flex justify-between items-center p-4 w-full">
      <div>
        <Button asChild variant="link">
          <Link href="/">
            <h1 className="text-primary">
              Connector <span className="text-muted-foreground">Factory</span>
            </h1>
          </Link>
        </Button>
        <Button asChild variant="link">
          <Link href="/thesis">Our thesis</Link>
        </Button>
        <Button asChild variant="link">
          <Link href="/specification">Specification</Link>
        </Button>
        <Button asChild variant="link">
          <Link href="/connectors">All connectors</Link>
        </Button>
      </div>
      <ThemeToggle />
    </header>
  );
}
