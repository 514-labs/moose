import {
  Section,
  Grid,
  ThirdWidthContentContainer,
} from "@514labs/design-system/components/containers";
import { Heading, Text } from "@514labs/design-system/typography";

export const FeaturesSection = () => {
  const content = {
    title: "Everything you need to build data-driven experiences",
    features: [
      {
        title: "Data models",
        description:
          "Define data models in your language, Moose derives the infrastructure for you",
      },
      {
        title: "Flows",
        description:
          "Write simple functions to transform and augment your data on the fly",
      },
      {
        title: "Aggregations & metrics",
        description:
          "Pivot, filter and group your data for repeatable insights and metrics",
      },
      {
        title: "Migrations",
        description:
          "Moose helps manage migrations for your end-to-end data  stack",
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
        title: "UI Components (coming soon)",
        description:
          "Embed insightful react components in your framework of choice",
      },
      {
        title: "Connectors & SDKs (coming soon)",
        description:
          "Connectors and auto generated SDKs get data in and out of moose",
      },
      {
        title: "Orchestration (coming soon)",
        description:
          "Configurable orchestration to make sure data gets where it needs to go reliably",
      },
    ],
  };

  return (
    <>
      <Section>
        <Grid className="mb-12 2xl:mb-20">
          <HalfWidthContentContainer className="lg:col-start-7">
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
