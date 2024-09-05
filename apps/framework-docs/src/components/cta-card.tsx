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
}

export function CTACard({
  title,
  description,
  ctaLink,
  ctaLabel,
  Icon,
  className,
}: CTACardProps) {
  return (
    <Card className={cn("md:w-64 w-auto h-auto rounded-3xl", className)}>
      <CardHeader>
        <div className="bg-gradient-to-b from-pink from-4.65% to-background to-93.24% w-fit rounded-[20px] border-transparent p-[2px]">
          <div className="rounded-[18px] w-fit bg-gradientDarkPink p-[2px]">
            <Icon className="m-3 h-[24px] w-[24px]" />
          </div>
        </div>
      </CardHeader>
      <CardContent>
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

export function CTACards({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex md:flex-row flex-col justify-start items-center gap-5 mt-5">
      {children}
    </div>
  );
}
