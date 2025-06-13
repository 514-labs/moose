import { cn } from "@/lib/utils";
import { LucideIcon } from "lucide-react";
import Link from "next/link";
import { ReactNode } from "react";

interface ValuePropCardProps {
  icon: LucideIcon;
  title: string;
  description: string;
  iconColor?: string;
  link?: { text: string; href: string; external?: boolean };
  className?: string;
}

export function ValuePropCard({
  icon,
  title,
  description,
  iconColor,
  link,
  className,
}: ValuePropCardProps) {
  const Icon = icon;
  return (
    <div
      className={cn(
        "flex items-center gap-3 p-3 rounded-lg border bg-card",
        className,
      )}
    >
      <Icon className={cn("w-6 h-6", iconColor || "text-blue-600")} />
      <div>
        <div className="font-medium text-sm">{title}</div>
        <div className="text-xs text-muted-foreground">{description}</div>
      </div>
      {link && (
        <Link href={link.href} target={link.external ? "_blank" : "_self"}>
          {link.text}
        </Link>
      )}
    </div>
  );
}

interface ValuePropCardsProps {
  children: ReactNode;
  columns?: 1 | 2 | 3;
  className?: string;
}

export function ValuePropCards({
  children,
  columns = 3,
  className,
}: ValuePropCardsProps) {
  const gridClasses = {
    1: "grid-cols-1",
    2: "grid-cols-1 md:grid-cols-2",
    3: "grid-cols-1 md:grid-cols-3",
  };

  return (
    <div className={cn("grid gap-4 my-6", gridClasses[columns], className)}>
      {children}
    </div>
  );
}
