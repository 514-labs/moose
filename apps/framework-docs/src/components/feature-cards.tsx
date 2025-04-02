import React, { ReactNode, FC } from "react";
import Link from "next/link";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export interface FeatureCardProps {
  href?: string;
  Icon: React.ElementType;
  title: string;
  description: string;
  features?: string[];
  variant?: "moose" | "aurora";
}

export function FeatureCard({
  href,
  Icon,
  title,
  description,
  features = [],
  variant = "moose",
}: FeatureCardProps) {
  const cardClasses = cn(
    "flex flex-col p-6 rounded-lg border border-border bg-card",
    {
      "transition-colors cursor-pointer": !!href,
      "hover:border-moose-purple hover:bg-moose-purple/10":
        !!href && variant === "moose",
      "hover:border-aurora-teal hover:bg-aurora-teal/10":
        !!href && variant === "aurora",
    },
  );

  const CardContent = () => (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-4">
        {Icon && (
          <div className="flex-shrink-0 self-start mt-1">
            <Icon className="h-[20px] w-[20px] text-moose-purple" />
          </div>
        )}
        <div>
          <h3 className="text-lg font-medium">{title}</h3>
          {description && (
            <div className="text-muted-foreground text-sm mt-1">
              {description}
            </div>
          )}
        </div>
      </div>
    </div>
  );

  if (href) {
    return (
      <Link href={href} className={cardClasses}>
        <CardContent />
      </Link>
    );
  }

  return (
    <div className={cardClasses}>
      <CardContent />
    </div>
  );
}

export interface FeatureGridProps {
  children: ReactNode;
  columns?: 1 | 2 | 3 | 4;
}

export function FeatureGrid({ children, columns = 2 }: FeatureGridProps) {
  const gridClass = {
    1: "grid-cols-1",
    2: "grid-cols-1 md:grid-cols-2",
    3: "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
    4: "grid-cols-1 md:grid-cols-2 lg:grid-cols-4",
  }[columns];

  return <div className={`grid ${gridClass} gap-6 my-8`}>{children}</div>;
}
