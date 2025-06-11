import { cn } from "@/lib/utils";
import { LucideIcon } from "lucide-react";

interface InlinePropItem {
  icon: LucideIcon;
  text: string;
  iconColor?: string;
}

interface InlinePropsProps {
  items: InlinePropItem[];
  className?: string;
}

export function InlineProps({ items, className }: InlinePropsProps) {
  return (
    <div
      className={cn(
        "flex items-center gap-4 my-4 text-sm text-primary",
        className,
      )}
    >
      {items.map((item, index) => (
        <div key={index} className="flex items-center gap-2">
          <item.icon
            className={cn(
              "w-4 h-4",
              `text-${item.iconColor}` || "text-primary",
            )}
          />
          <span className={cn(`text-${item.iconColor}` || "text-primary")}>
            {item.text}
          </span>
        </div>
      ))}
    </div>
  );
}
