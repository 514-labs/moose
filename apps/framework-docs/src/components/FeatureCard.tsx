import React, { ReactNode } from "react";
import Link from "next/link";

export interface FeatureCardProps {
  href: string;
  icon: ReactNode;
  title: string;
  description: string;
}

export function FeatureCard({
  href,
  icon,
  title,
  description,
}: FeatureCardProps) {
  return (
    <Link
      href={href}
      className="block group no-underline hover:no-underline transition-transform hover:scale-[1.02]"
    >
      <div className="flex items-start border border-border rounded-lg p-5 h-full hover:bg-muted/50 hover:border-accent-moo-purple transition-all">
        <div className="mr-3 mt-0.5 flex-shrink-0">{icon}</div>
        <div>
          <p className="text-primary font-medium">{title}</p>
          <p className="mt-2 text-muted-foreground">{description}</p>
        </div>
      </div>
    </Link>
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
