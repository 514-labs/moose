import React, { ReactNode } from "react";
import Link from "next/link";

export interface FeatureCardProps {
  href: string;
  icon: ReactNode;
  title: string;
  description: string;
  features?: string[];
}

export function FeatureCard({
  href,
  icon,
  title,
  description,
  features = [],
}: FeatureCardProps) {
  return (
    <Link
      href={href}
      className="block group no-underline hover:no-underline transition-transform hover:scale-[1.02]"
    >
      <div className="flex flex-col border border-border rounded-lg p-5 h-full hover:bg-muted/50 hover:border-accent-moo-purple transition-all">
        <div className="flex items-start">
          <div className="mr-3 mt-0.5 flex-shrink-0 text-moose-purple">
            {icon}
          </div>
          <div>
            <p className="text-primary font-medium">{title}</p>
            <p className="mt-2 text-muted-foreground">{description}</p>
          </div>
        </div>
        {features.length > 0 && (
          <div className="mt-4">
            <p className="text-sm text-muted-foreground">Features:</p>
            <ul className="list-disc list-inside text-sm mt-1">
              {features.map((feature) => (
                <li key={feature}>{feature}</li>
              ))}
            </ul>
          </div>
        )}
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
