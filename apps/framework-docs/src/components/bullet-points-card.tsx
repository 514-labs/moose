import { cn } from "@/lib/utils";
import { Check, X } from "lucide-react";

interface BasicBulletPoint {
  title: string;
  description?: string;
}

interface BulletPointCardProps {
  title: string;
  bullets: Array<BasicBulletPoint> | string[];
  className?: string;
  divider?: boolean;
  bulletStyle?: "default" | "number" | "check" | "x";
  uppercase?: boolean;
}

export function BulletPointsCard({
  title,
  bullets,
  className,
  divider = true,
  bulletStyle,
  uppercase = true,
}: BulletPointCardProps) {
  const renderBullet = (index: number) => {
    if (!bulletStyle) return null;

    const baseIconClasses = "shrink-0 mr-3";

    switch (bulletStyle) {
      case "default":
        return (
          <span
            className={cn(
              baseIconClasses,
              "inline-block w-2 h-2 bg-primary rounded-full mt-1.5",
            )}
          />
        );
      case "number":
        return (
          <span
            className={cn(
              baseIconClasses,
              "inline-flex items-center justify-center w-6 h-6 text-muted-foreground font-medium bg-primary/10 rounded-full text-sm",
            )}
          >
            {index + 1}
          </span>
        );
      case "check":
        return (
          <div
            className={cn(
              baseIconClasses,
              "p-1 bg-boreal-green/10 rounded-full",
            )}
          >
            <Check className="w-4 h-4 text-boreal-green" />
          </div>
        );
      case "x":
        return (
          <div
            className={cn(
              baseIconClasses,
              "p-1 bg-destructive/10 rounded-full",
            )}
          >
            <X className="w-4 h-4 text-destructive" />
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div
      className={cn(
        "rounded-xl border bg-card shadow-sm p-6 max-w-3xl my-6",
        className,
      )}
    >
      <h3
        className={cn(
          "font-mono text-sm tracking-wider mb-6 text-muted-foreground",
          uppercase && "uppercase",
        )}
      >
        {title}
      </h3>

      <div className="space-y-4">
        {bullets.map((point, index) => (
          <div key={index}>
            {divider && index > 0 && <div className="border-t my-4" />}
            <div className="py-1">
              <div className="flex items-start">
                {renderBullet(index)}
                <div className="flex-1 space-y-1">
                  <h4 className="font-medium text-primary">
                    {typeof point === "string" ? point : point.title}
                  </h4>
                  {typeof point === "object" && "description" in point && (
                    <p className="text-muted-foreground text-sm leading-relaxed">
                      {point.description}
                    </p>
                  )}
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
