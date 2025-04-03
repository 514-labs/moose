import React from "react";
import {
  Grid,
  HalfWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import {
  Heading,
  HeadingLevel,
  SmallText,
} from "@514labs/design-system-components/typography";

import { cn } from "@514labs/design-system-components/utils";
import { Card } from "@514labs/design-system-components/components";

interface ColumnProps {
  heading: string;
  body: string;
  list: string[];
  className: string;
}

export function Column({ heading, body, list, className }: ColumnProps) {
  return (
    <div className={cn("p-5 flex flex-col", className)}>
      <Heading level={HeadingLevel.l5} className="my-0 text-primary">
        {heading}
      </Heading>
      <SmallText className="h-1/2">{body}</SmallText>
      <ul className="text-muted-foreground">
        {list.map((listItem) => (
          <li>{listItem}</li>
        ))}
      </ul>
    </div>
  );
}

export function Columns({ children }: { children: React.ReactNode }) {
  return (
    <Card className="mt-4 md:gap-x-0">
      <Grid>{children}</Grid>
    </Card>
  );
}
