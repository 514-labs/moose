import { Badge } from "@/components/ui";
import { cn } from "@/lib/utils";

interface CustomBadgeProps {
  variant?: "moose" | "boreal" | "aurora" | "default";
  children: React.ReactNode;
  className?: string;
}

export function ProductBadge({
  variant = "default",
  children,
  className = "",
}: CustomBadgeProps) {
  const variantClasses = {
    moose:
      "bg-moose-purple hover:bg-moose-purple-dark text-moose-purple-foreground",
    boreal:
      "bg-boreal-green hover:bg-boreal-green-dark text-boreal-green-foreground",
    aurora: "bg-aurora-teal text-black",
    default: "",
  };

  return (
    <Badge className={`${variantClasses[variant]} ${className}`}>
      {children}
    </Badge>
  );
}

const variantColors = {
  moose: {
    background: "bg-purple-200 dark:bg-purple-950",
    foreground: "text-purple-800 dark:text-purple-400",
    stroke: "border-purple-300 dark:border-purple-900",
  },
  boreal: {
    background: "bg-green-200 dark:bg-green-950",
    foreground: "text-green-800 dark:text-green-400",
    stroke: "border-green-300 dark:border-green-900",
  },
  aurora: {
    background: "bg-teal-200 dark:bg-teal-950",
    foreground: "text-teal-800 dark:text-teal-400",
    stroke: "border-teal-300 dark:border-teal-900",
  },
  default: {
    background: "bg-muted",
    foreground: "text-muted-foreground",
    stroke: "border-border",
  },
};

interface IconBadgeProps {
  Icon?: React.ElementType;
  label: string;
  variant?: "moose" | "boreal" | "aurora" | "default";
}

export function IconBadge({ Icon, label, variant = "moose" }: IconBadgeProps) {
  return (
    <div
      className={cn(
        "flex items-center gap-3 w-fit border",
        variantColors[variant].background,
        variantColors[variant].stroke,
        variantColors[variant].foreground,
        Icon ? "rounded-full px-4 py-2" : "rounded-lg px-2 py-1",
      )}
    >
      {Icon && <Icon className="w-5 h-5" />}
      <span className="text-xs font-medium">{label}</span>
    </div>
  );
}
