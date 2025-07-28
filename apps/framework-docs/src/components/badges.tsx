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
    background: "bg-gray-950",
    foreground: "text-gray-100",
    stroke: "border-gray-800",
  },
  boreal: {
    background: "bg-gray-950",
    foreground: "text-gray-100",
    stroke: "border-gray-800",
  },
  aurora: {
    background: "bg-gray-950",
    foreground: "text-gray-100",
    stroke: "border-gray-800",
  },
  default: {
    background: "bg-gray-950",
    foreground: "text-gray-100",
    stroke: "border-gray-800",
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
        "flex items-center gap-1.5 w-fit border rounded-md text-xs font-medium",
        "bg-neutral-800 border-neutral-700 text-neutral-100",
        "px-2.5 py-1.5",
      )}
    >
      {Icon && <Icon className="w-3.5 h-3.5" />}
      <span>{label}</span>
    </div>
  );
}
