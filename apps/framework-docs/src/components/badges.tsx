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
    background: "purple-950",
    foreground: "purple-400",
    stroke: "purple-900",
  },
  boreal: {
    background: "green-950",
    foreground: "green-400",
    stroke: "green-900",
  },
  aurora: {
    background: "teal-950",
    foreground: "teal-400",
    stroke: "teal-900",
  },
  default: {
    background: "muted",
    foreground: "muted-foreground",
    stroke: "border",
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
        "flex items-center gap-3 w-fit",
        `bg-${variantColors[variant].background}`,
        `border-${variantColors[variant].stroke}`,
        `text-${variantColors[variant].foreground}`,
        Icon ? "rounded-full px-4 py-2" : "rounded-lg px-2 py-1",
      )}
    >
      {Icon && <Icon className="w-5 h-5" />}
      <span className="text-xs font-medium">{label}</span>
    </div>
  );
}
