import { cn } from "@/lib/utils";
import { Check, X } from "lucide-react";

interface BasicBulletPoint {
  title: string;
  description?: string;
}

interface BulletPointCardProps {
  title: string;
  bullets: Array<BasicBulletPoint>;
  className?: string;
  divider?: boolean;
  bulletStyle?: "default" | "number" | "check" | "x";
}

export function BulletPointsCard({
  title,
  bullets,
  className,
  divider = true,
  bulletStyle,
}: BulletPointCardProps) {
  const renderBullet = (index: number) => {
    if (!bulletStyle) return null;

    switch (bulletStyle) {
      case "default":
        return (
          <span className="inline-block w-2 h-2 bg-primary rounded-full mr-2 align-middle" />
        );
      case "number":
        return (
          <span className="inline-block text-sm text-primary font-medium mr-2">
            {index + 1}.
          </span>
        );
      case "check":
        return (
          <Check className="inline-block w-4 h-4 text-boreal-green mr-2 align-middle" />
        );
      case "x":
        return (
          <X className="inline-block w-4 h-4 text-muted-foreground mr-2 align-middle" />
        );
      default:
        return null;
    }
  };

  return (
    <div
      className={cn(
        "rounded-lg border bg-card shadow-sm p-6 max-w-3xl my-6",
        className,
      )}
    >
      <h3 className="font-mono uppercase mb-6 text-muted-foreground">
        {title}
      </h3>

      <div className="space-y-2">
        {bullets.map((point, index) => (
          <div key={index}>
            {divider && index > 0 && <div className="border-t my-2" />}
            <div className="py-1">
              <h4 className="font-medium text-primary mb-2 flex items-center">
                {renderBullet(index)}
                <span>{point.title}</span>
              </h4>
              <p className="text-muted-foreground">{point.description}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
