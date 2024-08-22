import React from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
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
  Icon: React.ElementType;
  className: string;
}

export function CTACard({
  title,
  description,
  ctaLink,
  Icon,
  className,
}: CTACardProps) {
  return (
    <Link href={ctaLink}>
      <Card className={cn("hover:bg-secondary md:w-64 w-auto h-56", className)}>
        <CardHeader>
          <div className="rounded-md w-fit bg-gradient">
            <Icon className="m-3 h-[24px] w-[24px]" />
          </div>
        </CardHeader>
        <CardContent>
          <Heading className="my-0 text-primary" level={HeadingLevel.l5}>
            {title}
          </Heading>
          <CardDescription className="mt-2">{description}</CardDescription>
        </CardContent>
      </Card>
    </Link>
  );
}
