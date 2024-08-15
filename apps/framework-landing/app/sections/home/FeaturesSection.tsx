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
import React from "react";

import { Box, Network, Share2, Terminal, Code2, Server } from "lucide-react";

export const FeaturesSection = () => {
  const content = {
    title: "Your tools, your workflows",
    subtitle:
      "Moose brings software developer ergonomics to the data & analytics stack",
    features: [
      {
        title: "Python and TypeScript",
        description:
          "Write code in your native language with your favorite IDE plug-ins",
        icon: <Code2 strokeWidth={1} />,
      },
      {
        title: "Local Dev Server",
        description:
          "Run your application locally and see the impact of code changes in real-time",
        icon: <Server strokeWidth={1} />,
      },
      {
        title: "Git-Based Workflows",
        description:
          "Integrate with existing version control and code collaboration workflows",
        icon: <Share2 strokeWidth={1} />,
      },
      {
        title: "OLAP Migrations",
        description:
          "Keep versions of your data synchronized through automated schema migrations",
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
              <ThirdWidthContentContainer
                key={index}
                className="flex flex-col gap-5 border p-5 rounded-3xl"
              >
                {feature.icon}
                <Text className="my-0">{feature.title}</Text>
                <SmallText className="my-0 text-muted-foreground text-[20px]">
                  {feature.description}
                </SmallText>
              </ThirdWidthContentContainer>
            );
          })}
        </Grid>
      </Section>
    </>
  );
};
