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

export function TwoColumns() {
  return (
    <Grid className="mt-4 md:gap-x-0 border rounded-3xl">
      <HalfWidthContentContainer className="border-r px-4 flex flex-col justify-between">
        <Heading level={HeadingLevel.l4}>
          Embed Data Features in User-Facing Applications
        </Heading>
        <SmallText>
          Moose can be combined with your favorite front-end framework to embed
          data-intensive features in your products.
        </SmallText>
        <ul className="text-muted-foreground mb-4">
          <li>Dynamic leaderboards</li>
          <li>Interactive charts</li>
          <li>Real-time metric feeds</li>
        </ul>
      </HalfWidthContentContainer>

      <HalfWidthContentContainer className="px-4 flex flex-col justify-between">
        <Heading level={HeadingLevel.l4}>
          Backend System for Enterprise Data Products
        </Heading>
        <SmallText>
          Create a backend system for enterprise data products, such as BI
          software and data exploration tools, to deliver interactive data
          applications.
        </SmallText>
        <ul className="text-muted-foreground mb-4">
          <li>Real-time Dashboards</li>
          <li>Observability & Log Monitoring</li>
          <li>Customer Data Platforms</li>
        </ul>
      </HalfWidthContentContainer>
    </Grid>
  );
}
