import {
  Section,
  Grid,
  ThirdWidthContentContainer,
  HalfWidthContentContainer,
} from "@514labs/design-system/components/containers";
import { Heading, Text } from "@514labs/design-system/typography";
import React from "react";

export const WhyMooseSection = () => {
  const content = {
    title:
      "We built MooseJS to make building data products easier for all devs",
    features: [
      {
        title: "Git-based workflows",
        description:
          "Git as your source of truth simplifies versioning and migrations",
      },
      {
        title: "Local dev server",
        description:
          "Build locally to make sure you can push to the cloud confidently",
      },
      {
        title: "Domain derived infra",
        description:
          "Infrastructure is automatically derived from your data models",
      },
      {
        title: "Test your way",
        description:
          "Use the testing frameworks you love to ensure high quality data",
      },
      {
        title: "Loose coupling",
        description:
          "Automatically keep producers and consumers loosely coupled",
      },
      {
        title: "Data primitives",
        description:
          "Simple, easy to configure primitives that ship with sane default",
      },
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

        <Grid>
          {content.features.map((feature, index) => {
            return (
              <ThirdWidthContentContainer key={index}>
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
