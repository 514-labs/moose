import React from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  Button,
} from "@514labs/design-system-components/components";
import Link from "next/link";
import {
  Heading,
  HeadingLevel,
  Text,
} from "@514labs/design-system-components/typography";
import { cn } from "@514labs/design-system-components/utils";

interface CTACardProps {
  title: string;
  description: string;
  ctaLink: string;
  ctaLabel: string;
  Icon: React.ElementType;
  className: string;
  variant: "default" | "gradient";
}

export function CTACard({
  title,
  description,
  ctaLink,
  ctaLabel,
  Icon,
  className,
  variant = "default",
}: CTACardProps) {
  return (
    <Card className={cn("rounded-3xl h-full flex flex-col", className)}>
      <CardHeader>
        <div
          className={cn(
            "w-fit rounded-[20px] p-[2px]",
            variant === "gradient"
              ? "bg-gradient-to-b from-pink from-4.65% to-background to-93.24% border-transparent"
              : "bg-muted",
          )}
        >
          <div
            className={cn(
              "rounded-[18px] w-fit p-[2px]",
              variant === "gradient" ? "bg-gradientDarkPink" : "bg-muted",
            )}
          >
            <Icon className="m-3 h-[24px] w-[24px] text-white" />
          </div>
        </div>
      </CardHeader>
      <CardContent className="flex-grow">
        <Heading className="my-0 text-primary" level={HeadingLevel.l5}>
          {title}
        </Heading>
        <CardDescription className="mt-2">{description}</CardDescription>
      </CardContent>
      <CardFooter>
        <Link href={ctaLink}>
          <Button className="font-normal">{ctaLabel}</Button>
        </Link>
      </CardFooter>
    </Card>
  );
}

interface CTACardsProps {
  children: React.ReactNode;
  columns?: number;
}

export function CTACards({ children, columns = 2 }: CTACardsProps) {
  const gridColumns = {
    1: "grid-cols-1",
    2: "grid-cols-1 md:grid-cols-2",
    3: "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
    4: "grid-cols-1 md:grid-cols-2 lg:grid-cols-4",
  };
  return (
    <div
      className={cn(
        "grid gap-5 mt-5",
        gridColumns[columns as keyof typeof gridColumns],
      )}
    >
      {children}
    </div>
  );
}
