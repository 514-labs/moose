import React, { ReactNode, FC } from "react";
import Link from "next/link";
import { cn } from "@/lib/utils";

export interface FeatureCardProps {
  href?: string;
  icon: ReactNode;
  title: string;
  description: string;
  features?: string[];
  variant?: "moose" | "aurora";
}

export function FeatureCard({
  href,
  icon,
  title,
  description,
  features = [],
  variant = "moose",
}: FeatureCardProps) {
  const cardClasses = cn("flex flex-col p-6 rounded-lg border border-border", {
    "transition-colors cursor-pointer": !!href,
    "hover:border-moose-purple hover:bg-moose-purple/10":
      !!href && variant === "moose",
    "hover:border-aurora-teal hover:bg-aurora-teal/10":
      !!href && variant === "aurora",
  });

  const CardContent = () => (
    <>
      {icon && <div className="mb-4">{icon}</div>}
      <h3 className="text-lg font-medium mb-2">{title}</h3>
      {description && (
        <div className="text-muted-foreground">{description}</div>
      )}
    </>
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
