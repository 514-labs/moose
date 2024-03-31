import {
  Section,
  Grid,
  FullWidthContentContainer,
  ThirdWidthContentContainer,
} from "design-system/components/containers";
import { Display, Heading, Text } from "design-system/typography";

export const FeaturesSection = () => {
  const content = {
    title: "Key Features",
    features: [
      {
        title: "Your tools, your way",
        description:
          "Git-based workflows, languages you know, and the IDE of choice. Moose is built to slot into your existing tools from your local env, to CI/CD and the cloud",
      },
      {
        title: "Auto migrations",
        description:
          "Moose automatically manages schema evolution across your data stack, so you don’t break upstream and downstream dependencies",
      },
      {
        title: "End-to-end typed",
        description:
          "Moose types all your components from data capture through consumption, so you catch issues at build time, instead of in production",
      },
      {
        title: "Built-in testing",
        description:
          "With Moose’s built-in testing framework, you can manage sample data and automatically test data pipelines as you’re developing",
      },
      {
        title: "Data product APIs",
        description:
          "Run stacks locally: see and test the impact of changes in real time as you edit code - just like developing a web app frontend",
      },
      {
        title: "Local dev server",
        description:
          "Run your whole data/analytics stack locally: see the impact of changes in real time as you edit code - just like developing a web application",
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
            <Heading>
              {content.features[0] && content.features[0].title}
            </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="xl:mb-5 xl:order-4">
            <Text>
              {content.features[0] && content.features[0].description}
            </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="mt-5 md:mt-5 xl:m-0 xl:order-2">
            <Heading>
              {content.features[1] && content.features[1].title}
            </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="md:mt-5 xl:m-0 xl:order-5 xl:mb-5">
            <Text>
              {content.features[1] && content.features[1].description}
            </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="md:mt-5 mt-5 xl:m-0 xl:order-3">
            <Heading>
              {content.features[2] && content.features[2].title}
            </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="md:mt-5  xl:m-0 xl:order-6 xl:mb-5">
            <Text>
              {content.features[2] && content.features[2].description}
            </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="mt-5 md:mt-5 xl:order-7 xl:m-0">
            <Heading>
              {content.features[3] && content.features[3].title}
            </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="md:mt-5  xl:m-0 xl:order-10">
            <Text>
              {content.features[3] && content.features[3].description}
            </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="mt-5 md:mb-5 md:mt-5  xl:m-0 xl:order-8">
            <Heading>
              {" "}
              {content.features[4] && content.features[4].title}{" "}
            </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="mb-5 md:mb-5 md:mt-5  xl:m-0 xl:order-11">
            <Text>
              {" "}
              {content.features[4] && content.features[4].description}{" "}
            </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="xl:order-9">
            <Heading>
              {" "}
              {content.features[5] && content.features[5].title}{" "}
            </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="xl:order-12">
            <Text>
              {" "}
              {content.features[5] && content.features[5].description}{" "}
            </Text>
          </ThirdWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
