import React from "react";
import { Card } from "@/components/ui/card";
import { Heading, Text } from "@/components/typography";
import { cn } from "@/lib/utils";

export interface TimelineStep {
  title: string;
  description?: string;
  icon?: React.ReactNode;
  learnMoreUrl?: string;
}

interface TimelineStepsProps {
  steps: TimelineStep[];
  className?: string;
  compact?: boolean;
}

export const TimelineSteps: React.FC<TimelineStepsProps> = ({
  steps,
  className,
  compact = false,
}) => {
  // Calculate color shade for each step (like the attached HTML)
  const getDotClass = (idx: number) => {
    const shades = [
      "bg-primary",
      "bg-primary/80",
      "bg-primary/60",
      "bg-primary/40",
      "bg-primary/20",
    ];
    // If more steps than shades, repeat the last shade
    return shades[idx] || shades[shades.length - 1];
  };

  return (
    <div className={cn("relative my-8", className)}>
      {/* Vertical line */}
      <div className="absolute left-6 top-0 bottom-0 w-0.5 bg-muted-foreground/20" />
      {steps.map((step, idx) => (
        <div
          key={idx}
          className={cn(
            "relative pl-12",
            idx !== steps.length - 1 ? "pb-8" : "",
            compact && "pb-4",
          )}
        >
          {/* Step dot/icon */}
          <div
            className={cn(
              "absolute left-4 w-4 h-4 -mt-2 rounded-full flex items-center justify-center",
              getDotClass(idx),
            )}
          >
            {step.icon && (
              <span className="text-white w-3 h-3 flex items-center justify-center">
                {step.icon}
              </span>
            )}
          </div>
          {/* Step content */}
          <h3 className={cn("font-medium mb-2", compact && "mb-1 text-base")}>
            {step.title}
          </h3>
          {step.description && (
            <p
              className={cn(
                "text-sm text-muted-foreground",
                compact && "text-xs mb-0",
              )}
            >
              {step.description}
            </p>
          )}
          {step.learnMoreUrl && (
            <a
              href={step.learnMoreUrl}
              className={cn(
                "inline-block mt-2 text-primary underline text-sm",
                compact && "text-xs",
              )}
              target="_blank"
              rel="noopener noreferrer"
            >
              Learn more
            </a>
          )}
        </div>
      ))}
    </div>
  );
};

export default TimelineSteps;
