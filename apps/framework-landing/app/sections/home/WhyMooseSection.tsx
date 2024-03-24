import {
  Section,
  Grid,
  FullWidthContentContainer,
  ThirdWidthContentContainer,
} from "design-system/components/containers";
import { Display, Heading, Text } from "design-system/typography";

export const WhyMooseSection = () => {
  const content = {
    title: "Why Moose?",
    valueProps: [
      {
        title: "Your data stack, unified",
        description:
          "Jobs, pipelines, streams, data models, tables, views, schemas, APIs, and SDKs -- no more coordinating a tangled web of individual components. With a framework-based approach, each component is aware of the bigger picture, so your data stack is easier to manage and more resilient to change.",
      },
      {
        title: "Best practices, unlocked",
        description:
          "Moose’s developer framework-based approach brings the standard software development workflow to the modern data stack, enabling best practices like: local development, rapid iteration, CI/CD, automated testing, version control, change management, code collaboration, DevOps, etc.",
      },
      {
        title: "Developers, delighted",
        description:
          "Moose simplifies the way you interact with you data and analytics stack to let you build rich data-intensive experiences using the languages you know, the application frameworks you love and the tools you leverage in your day-to-day workflow. Moose lets you focus on building your best app.",
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
          <ThirdWidthContentContainer className="   xl:order-1">
            <Heading>
              {content.valueProps[0] && content.valueProps[0].title}
            </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="   xl:order-4">
            <Text>
              {content.valueProps[0] && content.valueProps[0].description}
            </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="  mt-5  md:mt-5 md:mb-5 xl:m-0 xl:order-2">
            <Heading>
              {content.valueProps[1] && content.valueProps[1].title}
            </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="  mb-5  md:mt-5 md:mb-5 xl:m-0 xl:order-5">
            <Text>
              {content.valueProps[1] && content.valueProps[1].description}
            </Text>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="   xl:order-3">
            <Heading>
              {content.valueProps[2] && content.valueProps[2].title}
            </Heading>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="   xl:order-6">
            <Text>
              {content.valueProps[2] && content.valueProps[2].description}
            </Text>
          </ThirdWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
