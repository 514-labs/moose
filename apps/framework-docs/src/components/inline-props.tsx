import { cn } from "@/lib/utils";

interface InlinePropItem {
  Icon: React.ElementType;
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
          <item.Icon
            className={cn(
              "w-4 h-4",
              item.iconColor ? `text-${item.iconColor}` : "text-primary",
            )}
          />
          <span
            className={cn(
              item.iconColor ? `text-${item.iconColor}` : "text-primary",
            )}
          >
            {item.text}
          </span>
        </div>
      ))}
    </div>
  );
}
