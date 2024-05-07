import {
  Section,
  Grid,
  ThirdWidthContentContainer,
} from "@514labs/design-system/components/containers";
import { Text } from "@514labs/design-system/typography";

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
      <Section>
        <Grid>
          <HalfWidthContentContainer className="lg:col-start-7">
            <Heading> {content.title} </Heading>
          </HalfWidthContentContainer>
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
