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
  SmallText,
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
  cardName: string;
  variant: "default" | "gradient";
}

export function CTACard({
  title,
  description,
  ctaLink,
  ctaLabel,
  cardName,
  Icon,
  className,
  variant = "default",
}: CTACardProps) {
  return (
    <Card className={cn("h-full flex flex-col", className)}>
      <CardHeader>
        <div className="flex gap-2 items-center">
          <Icon className="h-[20px] w-[20px] text-purple-400" />
          <SmallText className="text-primary text-purple-400 my-0">
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
