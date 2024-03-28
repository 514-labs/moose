import { cn } from "design-system/utils";

export const PlaceholderImage = ({ className }: { className?: string }) => {
  return <div className={cn("relative ", className)}> </div>;
};
