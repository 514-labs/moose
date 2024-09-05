import React from "react";
import { cn } from "@514labs/design-system-components/utils";
import { SmallTextEmbed } from "@514labs/design-system-components/typography";
import { Lightbulb, StopCircle, FileWarning, PartyPopper } from "lucide-react";
import {
  Card,
  CardContent,
} from "@514labs/design-system-components/components";

interface CalloutProps {
  type: CalloutType;
  title?: string;
  children: React.ReactNode;
}

const calloutVariants = {
  success: {
    icon: <PartyPopper className="text-green" />,
    color: "bg-muted/20",
    border: "border-green/20",
    title: "Congrats!",
    titleColor: "text-green",
  },
  info: {
    icon: <Lightbulb className="text-muted-foreground" />,
    color: "bg-muted/20",
    border: "border-accent",
    title: "MooseTip:",
    titleColor: "text-muted-foreground",
  },
  warning: {
    icon: <StopCircle className="text-yellow" />,
    color: "bg-muted/20",
    border: "border-yellow/20",
    title: "Warning:",
    titleColor: "text-muted-foreground",
  },
  danger: {
    icon: <FileWarning className="text-descructive" />,
    color: "bg-muted/20",
    border: "border-destructive/20",
    title: "Error:",
    titleColor: "text-muted-foreground",
  },
};

type CalloutType = keyof typeof calloutVariants;

export function Callout({ type, title, children }: CalloutProps) {
  const variantProps = calloutVariants[type];

  return (
    <Card
      className={cn(
        "callout border b-[1px] rounded-3xl my-5 flex items-start w-full p-4 space-x-2",
        variantProps.color,
        variantProps.border,
      )}
    >
      <CardContent className="flex items-start space-x-2 p-0">
        <div className="flex-shrink-0 mt-1">{variantProps.icon}</div>
        <div className="flex-1">
          <SmallTextEmbed
            className={cn(
              "font-semibold my-0 inline-block mr-2",
              variantProps.titleColor,
            )}
          >
            {title || variantProps.title}
          </SmallTextEmbed>
          {children}
        </div>
      </CardContent>
    </Card>
  );
}
