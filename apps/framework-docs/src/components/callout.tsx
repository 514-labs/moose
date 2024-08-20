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

const typeToIcon = {
  info: <Info />,
  warning: <TriangleAlert />,
  danger: <Bug />,
};

type CalloutType = keyof typeof typeToIcon;

export default function Callout({ type, title, children }: CalloutProps) {
  const icon = typeToIcon[type];

  return (
    <Card
      className={cn(
        "border b-[1px] border-muted-foreground rounded-3xl my-5",
        type,
      )}
    >
      <CardHeader className="flex flex-row items-center justify-start gap-2 w-full my-0 pb-0">
        {icon && <div className="my-0 pt-0">{icon}</div>}
        <SmallText className="my-0 font-medium">{title}</SmallText>
      </CardHeader>
      <CardContent className="my-0">{children}</CardContent>
    </Card>
  );
}
