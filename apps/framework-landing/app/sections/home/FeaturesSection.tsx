import {
  ThirdWidthContentContainer,
  FullWidthContentContainer,
  Section,
  Grid,
} from "@/components/containers/page-containers";
import { Display, Heading, Text } from "@/components/typography/standard";

export const FeaturesSection = () => {
  const content = {
    title: "Key Features",
    features: [
      {
        title: "Git-based",
        description:
          "Git-based workflow enables software development best practices like version control and CI/CD",
      },
      {
        title: "Magical change managing",
        description:
          "Run stacks locally: see and test the impact of changes in real time as you edit code - just like developing a web app frontend",
      },
      {
        title: "End-to-end typed",
        description:
          "Run stacks locally: see and test the impact of changes in real time as you edit code - just like developing a web app frontend",
      },
      {
        title: "Built-in testing",
        description:
          "Git-based workflow enables software development best practices like version control and CI/CD",
      },
      {
        title: "Data product APIs",
        description:
          "Run stacks locally: see and test the impact of changes in real time as you edit code - just like developing a web app frontend",
      },
      {
        title: "Local dev server",
        description:
          "Run stacks locally: see and test the impact of changes in real time as you edit code - just like developing a web app frontend",
      },
    ],
  };

  return (
    <>
      <Section>
        <Grid>
          <FullWidthContentContainer>
            <Display> {content.title} </Display>
          </FullWidthContentContainer>
        </Grid>
      </Section>
      <Section>
        <Grid>
          <ThirdWidthContentContainer className="xl:order-1">
            <Heading> {content.features[0].title} </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="xl:mb-5 xl:order-4">
            <Text> {content.features[0].description} </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="mt-5 md:mt-5 xl:m-0 xl:order-2">
            <Heading> {content.features[1].title} </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="md:mt-5 xl:m-0 xl:order-5 xl:mb-5">
            <Text> {content.features[1].description} </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="md:mt-5 mt-5 xl:m-0 xl:order-3">
            <Heading> {content.features[2].title} </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="md:mt-5  xl:m-0 xl:order-6 xl:mb-5">
            <Text> {content.features[2].description} </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="mt-5 md:mt-5 xl:order-7 xl:m-0">
            <Heading> {content.features[3].title} </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="md:mt-5  xl:m-0 xl:order-10">
            <Text> {content.features[3].description} </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="mt-5 md:mb-5 md:mt-5  xl:m-0 xl:order-8">
            <Heading> {content.features[4].title} </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="mb-5 md:mb-5 md:mt-5  xl:m-0 xl:order-11">
            <Text> {content.features[4].description} </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="xl:order-9">
            <Heading> {content.features[5].title} </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="xl:order-12">
            <Text> {content.features[5].description} </Text>
          </ThirdWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
