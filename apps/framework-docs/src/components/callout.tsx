import React from "react";
import { cn } from "@514labs/design-system-components/utils";
import { SmallTextEmbed } from "@514labs/design-system-components/typography";
import { Info, TriangleAlert, Bug } from "lucide-react";
import {
  Card,
  CardContent,
  CardHeader,
} from "@514labs/design-system-components/components";

interface CalloutProps {
  type: CalloutType;
  title?: string;
  children: React.ReactNode;
}

const calloutVariants = {
  info: {
    icon: <Info className="text-muted-foreground" />,
    color: "bg-pink/10",
    border: "border-pink",
    title: "Moose Tip",
    titleColor: "text-muted-foreground",
  },
  warning: {
    icon: <TriangleAlert className="text-muted-foreground" />,
    color: "bg-yellow/10",
    border: "border-yellow",
    title: "Warning",
    titleColor: "text-muted-foreground",
  },
  danger: {
    icon: <Bug className="text-descructive" />,
    color: "bg-descructive/10",
    border: "border-destructive",
    title: "Troubleshooting Tip",
    titleColor: "text-destructive",
  },
};

type CalloutType = keyof typeof calloutVariants;

export function Callout({ type, title, children }: CalloutProps) {
  const variantProps = calloutVariants[type];

  return (
    <Card
      className={cn(
        "border b-[1px] rounded-3xl my-5 flex items-start w-full p-4 space-x-2 ",
        variantProps.color,
        variantProps.border,
      )}
    >
      <div className="flex-shrink-0">{variantProps.icon}</div>
      <div className="flex-1">
        <SmallTextEmbed
          className={`font-semibold my-0 ${variantProps.titleColor}`}
        >
          {title ? title : variantProps.title}
        </SmallTextEmbed>
        {children}
      </div>
    </Card>
  );
}
