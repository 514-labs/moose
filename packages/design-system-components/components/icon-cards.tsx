import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "./ui/card";

import { Button } from "./ui/button";
import Link from "next/link";
import {
  Heading,
  HeadingLevel,
  Text,
} from "@514labs/design-system-components/typography";
import { cn } from "../lib/utils";
import React from "react";

interface IconCardProps {
  title: string;
  description: string;
  Icon: React.ElementType;
  className?: string;
  variant?: "default" | "gradient";
}

interface IconPops {
  Icon: React.ElementType;
  variant: "default" | "gradient";
}

interface CTACardProps extends IconCardProps {
  ctaLink: string;
  ctaLabel: string;
}

interface BaseCardProps extends IconCardProps {
  children?: React.ReactNode;
}

function BaseCard({
  title,
  description,
  Icon,
  className = "",
  variant = "default",
  children,
}: BaseCardProps) {
  return (
    <Card className={cn("rounded-2xl h-full flex flex-col", className)}>
      <CardHeader>
        <BackgroundIcon variant={variant} Icon={Icon} />
      </CardHeader>
      <CardContent className="flex-grow">
        <Text className="my-0">{title}</Text>
        <Text className="mt-2 text-muted-foreground">{description}</Text>
      </CardContent>
      {children}
    </Card>
  );
}

export function BackgroundIcon({ Icon, variant = "default" }: IconPops) {
  return (
    <div
      className={cn(
        "w-fit rounded-[12px] p-[2px]",
        variant === "gradient"
          ? "bg-gradient-to-b from-pink from-4.65% to-background to-93.24% border-transparent"
          : "",
      )}
    >
      <div
        className={cn(
          "rounded-[10px] w-fit p-2",
          variant === "gradient" ? "bg-gradientDarkPink" : "bg-[#262626]",
        )}
      >
        <Icon className="h-[24px] w-[24px] text-white" />
      </div>
    </div>
  );
}

export function IconCard({
  title,
  description,
  Icon,
  className,
  variant,
}: IconCardProps) {
  return (
    <BaseCard
      title={title}
      description={description}
      Icon={Icon}
      className={className}
      variant={variant}
    />
  );
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
    <BaseCard
      title={title}
      description={description}
      Icon={Icon}
      className={className}
      variant={variant}
    >
      <CardFooter>
        <Link href={ctaLink}>
          <Button>{ctaLabel}</Button>
        </Link>
      </CardFooter>
    </BaseCard>
  );
}
