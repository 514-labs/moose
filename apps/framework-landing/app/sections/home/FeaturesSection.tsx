import {
  Section,
  Grid,
  ThirdWidthContentContainer,
  FullWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import {
  Heading,
  Text,
  SmallText,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import { IconCard } from "@514labs/design-system-components/components";
import React from "react";

import {
  Box,
  Network,
  Share2,
  Terminal,
  Code2,
  Server,
  FileCheck,
  Eye,
  Database,
  HardDrive,
  GitFork,
} from "lucide-react";

export const FeaturesSection = () => {
  const content = {
    title: "Features",
    subtitle: "",
    features: [
      {
        title: "Local & Production Parity",
        description:
          "Run and test your entire app locally, then deploy with a single command",
        icon: HardDrive,
      },
      {
        title: "Built-In Infrastructure",
        description:
          "Leverage ClickHouse for analytics storage and Redpanda for data streaming",
        icon: GitFork,
      },
      {
        title: "Powerful CLI",
        description:
          "Interact with your app primitives and infra without leaving your terminal",
        icon: Terminal,
      },
      {
        title: "Schema Versioning",
        description:
          "Set up automated migrations and version syncs for schema changes",
        icon: Database,
      },
      {
        title: "Task Orchestration",
        description:
          "Manage recurring jobs and batch operations through a cron scheduler",
        icon: FileCheck,
      },
      {
        title: "Monitoring & Observability",
        description:
          "Keep tabs on app performance with built-in metrics and logging",
        icon: Eye,
      },
    ],
  };

  return (
    <>
      <Section className="mx-auto max-w-5xl">
        <Grid className="mb-12 2xl:mb-20">
          <FullWidthContentContainer>
            <Heading
              level={HeadingLevel.l1}
              className="max-w-5xl justify-center align-center text-center sm:text-5xl"
            >
              {content.title}
            </Heading>
            <Heading
              level={HeadingLevel.l2}
              className="max-w-5xl justify-center align-center text-center mb-24 text-muted-foreground"
            >
              {content.subtitle}
            </Heading>
          </FullWidthContentContainer>
        </Grid>
        <Grid className="gap-y-10">
          {content.features.map((feature, index) => {
            return (
              <ThirdWidthContentContainer key={index} className="flex flex-col">
                <IconCard
                  title={feature.title}
                  Icon={feature.icon}
                  description={feature.description}
                />
              </ThirdWidthContentContainer>
            );
          })}
        </Grid>
      </Section>
    </>
  );
};
