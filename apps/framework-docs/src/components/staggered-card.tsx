import { cn } from "@/lib/utils";
import {
  Card,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
  CardHeader,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import Link from "next/link";

interface StaggeredCardProps {
  children: React.ReactNode;
  stagger?: "left" | "right";
  className?: string;
}

export function StaggeredCard({
  children,
  stagger = "left",
  className,
}: StaggeredCardProps) {
  return (
    <div>
      <div
        className={cn(
          "grid grid-cols-1 gap-4 mx-4 items-start pt-8",
          stagger === "left" ?
            "md:grid-cols-[40%_60%]"
          : "md:grid-cols-[60%_40%]",
          className,
        )}
      >
        {children}
      </div>
    </div>
  );
}

interface StaggeredContentProps {
  title?: string;
  description?: string;
  cta?: {
    label: string;
    href: string;
    variant?: "default" | "outline" | "link";
  };
}

export function StaggeredContent({
  title,
  description,
  cta,
}: StaggeredContentProps) {
  return (
    <div>
      <CardHeader className="px-0 pt-0">
        <h3 className="text-2xl">{title}</h3>
      </CardHeader>
      <CardContent className="px-0">
        <CardDescription>{description}</CardDescription>
      </CardContent>
      <CardFooter className="px-0">
        {cta && (
          <Button asChild variant={cta.variant}>
            <Link href={cta.href}>{cta.label}</Link>
          </Button>
        )}
      </CardFooter>
    </div>
  );
}

interface StaggeredCodeProps {
  children: React.ReactNode;
  language?: string;
}

export function StaggeredCode({
  children,
  language = "ts",
}: StaggeredCodeProps) {
  return <div className="rounded-lg overflow-x-auto pr-4">{children}</div>;
}
