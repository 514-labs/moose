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
import {
  FileBarChart,
  GitBranch,
  LayoutDashboard,
  PieChart,
} from "lucide-react";

export const WhatIsMooseFor = () => {
  return (
    <>
      <Section className="max-w-5xl mx-auto">
        <FullWidthContentContainer>
          <Heading
            level={HeadingLevel.l1}
            className="max-w-5xl justify-center align-center text-center mb-24 sm:text-5xl"
          >
            Use Cases
          </Heading>
        </FullWidthContentContainer>
        <Grid className="grid grid-cols-2 grid-rows-2 grid-flow-col">
          <IconCard
            title="Real-Time Dashboards"
            description=""
            Icon={LayoutDashboard}
          />
          <IconCard
            title="Interactive Analytics Features"
            description=""
            Icon={PieChart}
          />
          <IconCard
            title="Enterprise Data Platforms"
            description=""
            Icon={FileBarChart}
          />
          <IconCard
            title="Custom Data Pipelines"
            description=""
            Icon={GitBranch}
          />
        </Grid>
      </Section>
    </>
  );
};
