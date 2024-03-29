import { Heading, Text } from "design-system/typography";
import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";

export const BuiltOnSection = () => {
  const content = {
    title: "Build for speed, built for scale, built for you",
    description:
      "Moose comes batteries included with a best-in-class data stack. Performant, scalable, reliable and, above all, runs on your laptop, this stack can handle different types of realtime and batch workloads. From high-heat shoe launches to bio informatics and everything else in between, moose has you covered.",
    stack: [
      {
        tag: "01",
        title: "Rust",
        description:
          "Because speed and safety are paramount. Rust powers the core of the moose framework and helps deliver unparraleled performance and reliability.",
      },
      {
        tag: "02",
        title: "Redpanda",
        description:
          "Best in class performance, lightweight enough to run on your machine and a kafka compliant API. What's not to love?",
      },
      {
        tag: "03",
        title: "Clickhouse",
        description:
          "Lightning fast, columnar storage, and a SQL interface. Clickhouse is the perfect database for your large scale OLAP workloads.",
      },
      {
        tag: "coming soon",
        title: "DuckDB",
        description:
          "For when you need to keep things fast and local. Duck DB is the perfect database for your local first data-intensive applications.",
      },
    ],
  };
  return (
    <Section>
      <Grid className="gap-y-5">
        <HalfWidthContentContainer>
          <Heading> {content.title} </Heading>
        </HalfWidthContentContainer>
        <HalfWidthContentContainer>
          <Text> {content.description} </Text>
        </HalfWidthContentContainer>
      </Grid>

      {content.stack.map((stackItem, index) => {
        return (
          <Grid className="my-5" key={index}>
            <HalfWidthContentContainer>
              <Text className="my-0"> {stackItem.tag} </Text>
            </HalfWidthContentContainer>
            <HalfWidthContentContainer>
              <Heading className="mt-0"> {stackItem.title} </Heading>
              <Text> {stackItem.description} </Text>
            </HalfWidthContentContainer>
          </Grid>
        );
      })}
    </Section>
  );
};
