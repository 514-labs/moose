import React from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  Button,
} from "@/components/ui";
import Link from "next/link";
import {
  Heading,
  HeadingLevel,
  SmallText,
  Text,
} from "@/components/typography";
import { cn } from "@/lib/utils";
import { IconBadge } from "@/components/badges";
import { ArrowRight } from "lucide-react";

interface CTACardProps {
  title: string;
  description: string;
  ctaLink: string;
  ctaLabel: string;
  Icon?: React.ElementType;
  badge?: {
    variant: "boreal" | "aurora" | "moose" | "default";
    text: string;
  };
  className?: string;
  cardName?: string;
  variant?: "default" | "gradient" | "aurora";
  orientation?: "vertical" | "horizontal";
}

export function CTACard({
  title,
  description,
  ctaLink,
  ctaLabel,
  cardName,
  Icon,
  badge,
  className = "",
  variant = "default",
  orientation = "vertical",
}: CTACardProps) {
  return orientation == "horizontal" ?
      <Link href={ctaLink} className={cn("w-full", className)}>
        <Card
          className={cn("h-full flex items-center hover:bg-muted transition")}
        >
          {badge ?
            <IconBadge variant={badge.variant} label={badge.text} />
          : Icon ?
            <div className="ml-6 bg-muted rounded-lg p-4 shrink-0 flex items-center justify-center border border-neutral-200 dark:border-neutral-800">
              <Icon
                className={cn(
                  "h-6 w-6",
                  variant === "aurora" ? "text-aurora-teal" : "text-primary",
                )}
              />
            </div>
          : null}
          <CardContent className="flex-1 min-w-0 pl-6">
            <Heading className="text-primary mb-0" level={HeadingLevel.l5}>
              {title}
            </Heading>
            <CardDescription className="mt-2">{description}</CardDescription>
          </CardContent>
          <div className="mr-6 rounded-lg p-4 shrink-0 flex items-center justify-center">
            <ArrowRight className="h-6 w-6" />
            <span className="sr-only">{ctaLabel}</span>
          </div>
        </Card>
      </Link>
    : <Card className={cn("h-full flex flex-col", className)}>
        <CardHeader>
          <div className="flex gap-2 items-center">
            {badge ?
              <IconBadge variant={badge.variant} label={badge.text} />
            : orientation === "vertical" && Icon ?
              <div className="bg-muted rounded-lg p-4 border border-neutral-200 dark:border-neutral-800">
                <Icon
                  className={cn(
                    "h-6 w-6",
                    variant === "aurora" ? "text-aurora-teal" : "text-primary",
                  )}
                />
              </div>
            : null}
          </div>
        </CardHeader>
        <CardContent>
          <Heading className="text-primary mb-0" level={HeadingLevel.l5}>
            {title}
          </Heading>
          <CardDescription className="mt-2">{description}</CardDescription>
        </CardContent>
        <CardFooter>
          <Link href={ctaLink}>
            <Button className="font-normal" variant="secondary">
              {ctaLabel}
            </Button>
          </Link>
        </CardFooter>
      </Card>;
}

interface CTACardsProps {
  children: React.ReactNode;
  columns?: number;
  rows?: number;
}

export function CTACards({ children, columns = 2, rows = 1 }: CTACardsProps) {
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
        `grid-rows-${rows}`,
      )}
    >
      {children}
    </div>
  );
}
