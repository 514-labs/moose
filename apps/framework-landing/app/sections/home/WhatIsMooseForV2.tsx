import {
  FullWidthContentContainer,
  Grid,
  Section,
} from "@514labs/design-system-components/components/containers";

import {
  Heading,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import React from "react";

import { IconCard } from "@514labs/design-system-components/components";
import { Code, GitCompare, LayoutDashboard, PieChart } from "lucide-react";

export const WhatIsMooseFor = () => {
  return (
    <Section className="max-w-5xl mx-auto">
      <FullWidthContentContainer>
        <Heading
          level={HeadingLevel.l1}
          className="max-w-5xl justify-center align-center text-center mb-24 sm:text-5xl"
        >
          Use Cases.{" "}
          <span className="text-muted-foreground">
            What Moose is really great for.
          </span>
        </Heading>
      </FullWidthContentContainer>
      <Grid className="grid grid-cols-2 grid-rows-2 grid-flow-col">
        <IconCard
          title="Interactive Analytics Features"
          description="Enable embedded personalized charts, metrics, and data feeds in your user-facing applications"
          Icon={PieChart}
        />
        <IconCard
          title="Custom Data APIs"
          description="Serve processed data through authenticated APIs for external applications"
          Icon={Code}
        />
        <IconCard
          title="Enterprise Data Products"
          description="Power reports, dashboards, and internal analytics like C360, observability, or supply chain"
          Icon={LayoutDashboard}
        />
        <IconCard
          title="Real-Time Processing Pipelines"
          description="Process and analyze live data streams for real-time insights and workflow automation"
          Icon={GitCompare}
        />
      </Grid>
    </Section>
  );
};
