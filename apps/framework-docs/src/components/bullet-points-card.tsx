import { cn } from "@/lib/utils";

interface BulletPointCardProps {
  title: string;
  bullets: Array<{
    title: string;
    description: string;
  }>;
  className?: string;
}

export function BulletPointsCard({
  title,
  bullets,
  className,
}: BulletPointCardProps) {
  return (
    <div
      className={cn(
        "rounded-lg border bg-card shadow-sm p-6 max-w-3xl",
        className,
      )}
    >
      <h3 className="font-mono uppercase mb-6 text-muted-foreground">
        {title}
      </h3>

      <div className="space-y-2">
        {bullets.map((point, index) => (
          <div key={index}>
            {index > 0 && <div className="border-t my-2" />}
            <div className="py-2">
              <h4 className="font-medium text-primary mb-2">{point.title}</h4>
              <p className="text-muted-foreground">{point.description}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
