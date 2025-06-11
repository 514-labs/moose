import { cn } from "@/lib/utils";
import { LucideIcon } from "lucide-react";
import Link from "next/link";

interface ValuePropItem {
  icon: LucideIcon;
  title: string;
  description: string;
  iconColor?: string;
  link?: { text: string; href: string; external?: boolean };
}

interface ValuePropCardsProps {
  items: ValuePropItem[];
  columns?: 1 | 2 | 3;
  className?: string;
}

export function ValuePropCards({
  items,
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
      {items.map((item, index) => (
        <div
          key={index}
          className="flex items-center gap-3 p-3 rounded-lg border bg-card"
        >
          <item.icon
            className={cn("w-6 h-6", item.iconColor || "text-blue-600")}
          />
          <div>
            <div className="font-medium text-sm">{item.title}</div>
            <div className="text-xs text-muted-foreground">
              {item.description}
            </div>
          </div>
          {item.link && (
            <Link
              href={item.link.href}
              target={item.link.external ? "_blank" : "_self"}
            >
              {item.link.text}
            </Link>
          )}
        </div>
      ))}
    </div>
  );
}
