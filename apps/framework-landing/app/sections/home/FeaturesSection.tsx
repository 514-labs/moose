import {
  Section,
  Grid,
  ThirdWidthContentContainer,
  HalfWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import { Heading, Text } from "@514labs/design-system-components/typography";
import React from "react";

import { Box, Database, Layers, Network, Share2, Trello } from "lucide-react";

export const FeaturesSection = () => {
  const content = {
    title: "Everything you need to build data-driven experiences",
    features: [
      {
        title: "Data Models",
        description:
          "Define data models in your language, Moose derives the infrastructure for you",
        icon: <Layers strokeWidth={1} />,
      },
      {
        title: "Flows",
        description:
          "Write simple functions to transform and augment your data on the fly",
        icon: <Share2 strokeWidth={1} />,
      },
      {
        title: "Aggregations",
        description:
          "Pivot, filter and group your data for repeatable insights and metrics",
        icon: <Network strokeWidth={1} />,
      },
      {
        title: "Migrations",
        description:
          "Moose helps manage migrations for your end-to-end data stack",
        icon: <Database strokeWidth={1} />,
      },
      {
        title: "Templates",
        description:
          "Get up and running in seconds with pre-built application templates",
        icon: <Trello strokeWidth={1} />,
      },
      {
        title: "Packaging",
        description:
          "Easily package your application for deployment in any environment",
        icon: <Box strokeWidth={1} />,
      },
      // {
      //   title: "UI Components (coming soon)",
      //   description:
      //     "Embed insightful react components in your framework of choice",
      // },
      // {
      //   title: "Connectors & SDKs (coming soon)",
      //   description:
      //     "Connectors and auto generated SDKs get data in and out of moose",
      // },
      // {
      //   title: "Orchestration (coming soon)",
      //   description:
      //     "Configurable orchestration to make sure data gets where it needs to go reliably",
      // },
    ],
  };

  return (
    <>
      <Section className="mx-auto xl:max-w-screen-xl">
        <Grid className="mb-12 2xl:mb-20">
          <HalfWidthContentContainer>
            <Heading> {content.title} </Heading>
          </HalfWidthContentContainer>
        </Grid>
        <Grid className="gap-y-10">
          {content.features.map((feature, index) => {
            return (
              <ThirdWidthContentContainer
                key={index}
                className="flex flex-col gap-2"
              >
                {feature.icon}
                <Text className="my-0">{feature.title}</Text>
                <Text className="my-0 text-muted-foreground">
                  {feature.description}
                </Text>
              </ThirdWidthContentContainer>
            );
          })}
        </Grid>
      </Section>
    </>
  );
};
