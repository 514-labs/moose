import React from "react";
import { cn } from "@514labs/design-system-components/utils";
import { SmallText } from "@514labs/design-system-components/typography";
import { Info, TriangleAlert, Bug } from "lucide-react";
import {
  Card,
  CardContent,
  CardHeader,
} from "@514labs/design-system-components/components";

interface CalloutProps {
  type: CalloutType;
  title: string;
  children: React.ReactNode;
}

const calloutVariants = {
  info: {
    icon: <Info />,
    color: "bg-blue/10",
    border: "border-blue",
  },
  warning: {
    icon: <TriangleAlert />,
    color: "bg-yellow/10",
    border: "border-yellow",
  },
  danger: {
    icon: <Bug />,
    color: "bg-descructive/10",
    border: "border-destructive",
  },
};

type CalloutType = keyof typeof calloutVariants;

export function Callout({ type, title, children }: CalloutProps) {
  const variantProps = calloutVariants[type];

  return (
    <Card
      className={cn(
        "border b-[1px] rounded-3xl my-5",
        variantProps.color,
        variantProps.border,
      )}
    >
      <CardHeader className="flex flex-row items-center justify-start gap-2 w-full my-0 pb-0">
        <div className="my-0 pt-0">{variantProps.icon}</div>
        <SmallText className="my-0 font-medium">{title}</SmallText>
      </CardHeader>
      <CardContent className="my-0">{children}</CardContent>
    </Card>
  );
}
