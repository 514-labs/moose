import * as React from "react";
import { Badge } from "@/components/ui";

interface CustomBadgeProps {
  variant?: "moose" | "boreal" | "aurora" | "default";
  children: string | JSX.Element | JSX.Element[];
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
