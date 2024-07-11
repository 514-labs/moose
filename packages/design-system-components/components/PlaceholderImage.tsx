import { cn } from "../lib/utils";

export const PlaceholderImage = ({ className }: { className?: string }) => {
  return <div className={cn("relative ", className)}> </div>;
};
