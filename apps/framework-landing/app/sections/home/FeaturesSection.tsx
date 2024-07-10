import {
  Section,
  Grid,
  ThirdWidthContentContainer,
  HalfWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import {
  Heading,
  Text,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import React from "react";

import { Box, Database, Network, Share2, Terminal, Code2 } from "lucide-react";

export const FeaturesSection = () => {
  const content = {
    title: "Your tools, your workflows",
    subtitle:
      "Moose brings the ergonomics of web development frameworks to the data & analytics stack",
    features: [
      {
        title: "Python and TypeScript",
        description:
          "Write code in your native language with your favorite IDE plug-ins and AI assistants",
        icon: <Code2 strokeWidth={1} />,
      },
      {
        title: "Local Dev Server",
        description:
          "Run your application locally and see the impact of code changes in real-time",
        icon: <Database strokeWidth={1} />,
      },
      {
        title: "Git-Based Workflows",
        description:
          "Integrate with existing version control and code collaboration workflows",
        icon: <Share2 strokeWidth={1} />,
      },
      {
        title: "Migrations",
        description:
          "Keep versions your data synchronized through automated schema migrations",
        icon: <Network strokeWidth={1} />,
      },
      {
        title: "Powerful CLI",
        description:
          "Use terminal commands to automate setup and build processes",
        icon: <Terminal strokeWidth={1} />,
      },
      {
        title: "Deploy with Docker",
        description:
          "Package your application for deployment in any environment from the CLI",
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
            <Heading level={HeadingLevel.l3} className="text-muted-foreground">
              {content.subtitle}
            </Heading>
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
