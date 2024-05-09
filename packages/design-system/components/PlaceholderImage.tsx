import { cn } from "@514labs/design-system/utils";

export const PlaceholderImage = ({ className }: { className?: string }) => {
  return <div className={cn("relative ", className)}> </div>;
};
