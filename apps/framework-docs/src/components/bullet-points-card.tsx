import { cn } from "@/lib/utils";
import { Check, X } from "lucide-react";

interface BasicBulletPoint {
  title: string;
  description?: string;
  link?: {
    text: string;
    href: string;
    external?: boolean;
  };
}

interface BulletPointCardProps {
  title: string;
  bullets: Array<BasicBulletPoint> | string[];
  className?: string;
  divider?: boolean;
  bulletStyle?: "default" | "number" | "check" | "x";
  uppercase?: boolean;
  compact?: boolean;
}

export function BulletPointsCard({
  title,
  bullets,
  className,
  divider = true,
  bulletStyle,
  uppercase = true,
  compact = false,
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
        "rounded-xl border bg-card shadow-sm max-w-3xl my-6",
        compact ? "p-4" : "p-6",
        className,
      )}
    >
      <h3
        className={cn(
          "font-mono text-sm tracking-wider text-muted-foreground",
          compact ? "mb-3" : "mb-6",
          uppercase && "uppercase",
        )}
      >
        {title}
      </h3>

      <div className={compact ? "space-y-2" : "space-y-4"}>
        {bullets.map((point, index) => (
          <div key={index}>
            {divider && index > 0 && (
              <div className={cn("border-t", compact ? "my-2" : "my-4")} />
            )}
            <div className={compact ? "py-0.5" : "py-1"}>
              <div className="flex items-start">
                {renderBullet(index)}
                <div className="flex-1 space-y-1">
                  <h4
                    className={cn(
                      "font-medium text-primary",
                      compact && "text-sm",
                    )}
                  >
                    {typeof point === "string" ? point : point.title}
                  </h4>
                  {typeof point === "object" && "description" in point && (
                    <div
                      className={cn(
                        "text-muted-foreground leading-relaxed",
                        compact ? "text-xs" : "text-sm",
                      )}
                    >
                      <p className="mb-1">{point.description}</p>
                      {point.link && (
                        <a
                          href={point.link.href}
                          className="text-blue-600 hover:text-blue-800 hover:underline transition-colors inline-flex items-center gap-1"
                          target={point.link.external ? "_blank" : undefined}
                          rel={
                            point.link.external ?
                              "noopener noreferrer"
                            : undefined
                          }
                        >
                          {point.link.text}
                          {point.link.external && (
                            <svg
                              className="w-3 h-3"
                              fill="none"
                              stroke="currentColor"
                              viewBox="0 0 24 24"
                            >
                              <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                              />
                            </svg>
                          )}
                        </a>
                      )}
                    </div>
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
