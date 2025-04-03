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
import { ProductBadge } from "@/components/product-badge";

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
  variant?: "default" | "gradient";
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
}: CTACardProps) {
  return (
    <Card className={cn("h-full flex flex-col", className)}>
      <CardHeader>
        <div className="flex gap-2 items-center">
          {badge ? (
            <ProductBadge variant={badge.variant}>{badge.text}</ProductBadge>
          ) : Icon ? (
            <Icon className="h-[20px] w-[20px] text-moose-purple" />
          ) : null}
          <SmallText className="text-primary text-moose-purple my-0">
            {cardName}
          </SmallText>
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
