import {
  Section,
  Grid,
  FullWidthContentContainer,
  ThirdWidthContentContainer,
} from "@514labs/design-system/components/containers";
import { Heading, Text } from "@514labs/design-system/typography";

export const FeaturesSection = () => {
  const content = {
    title: "Everything you need to build data-driven experiences",
    features: [
      {
        title: "Data Models",
        description:
          "Define data models in your language, Moose derives the rest",
      },
      {
        title: "Flows",
        description: "Write simple functions to transform your data on the fly",
      },
      {
        title: "Aggregations & metrics",
        description:
          "Pivot, filter amd group your data for repeatable insights and metrics",
      },
      {
        title: "Migrations",
        description:
          "Moose helps manage migrations for your end-to-end data  stack",
      },
      {
        title: "UI Components",
        description:
          "Embed insightful react components in your framework of choice",
      },
      {
        title: "Connectors & SDKs",
        description:
          "Connectors and auto generated SDKs get data in and out of moose",
      },
      {
        title: "Templates",
        description:
          "Get up and running in seconds with pre-built application templates",
      },
      {
        title: "Packaging",
        description:
          "Easily package your application for deployment in any environment",
      },
      {
        title: "Orchestration",
        description:
          "Configurable orchestration to make sure things reliably happen",
      },
    ],
  };

  return (
    <>
      <Section>
        <Grid>
          <FullWidthContentContainer>
            <Heading> {content.title} </Heading>
          </FullWidthContentContainer>
        </Grid>
      </Section>
      <Section>
        <Grid className="gap-y-5">
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
