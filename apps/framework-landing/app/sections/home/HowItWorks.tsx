import {
  FullWidthContentContainer,
  Grid,
  Section,
  ThirdWidthContentContainer,
} from "@514labs/design-system/components/containers";
import { Heading, Text } from "@514labs/design-system/typography";
import React from "react";

const content = {
  title: "How it works",
  steps: [
    {
      title: "Create data products in TypeScript or Python",
      description:
        "Use primitives, such as data models, flows and aggregations to quickly implement logic",
    },
    {
      title: "Moose automatically provisions your data stack",
      description:
        "As you build your application, Moose provisions topics, tables and ingestion points locally",
    },
    {
      title: "Derive insights and make them available to users",
      description:
        "Use your favorite react framework or BI tools to build rich experiences on top of Moose",
    },
  ],
};

export const HowItWorksSection = () => {
  return (
    <>
      <Section>
        <Grid>
          <FullWidthContentContainer>
            <Heading>{content.title}</Heading>
          </FullWidthContentContainer>
        </Grid>
      </Section>

      <Section>
        <Grid className="gap-5">
          {content.steps.map((step, index) => {
            return (
              <ThirdWidthContentContainer key={index}>
                <Text className="my-0">{step.title}</Text>
                <Text className="my-0 text-muted-foreground">
                  {step.description}
                </Text>
              </ThirdWidthContentContainer>
            );
          })}
        </Grid>
      </Section>
    </>
  );
};
