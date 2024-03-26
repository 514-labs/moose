import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";
import { PlaceholderImage } from "../../page";
import { Heading, Text } from "design-system/typography";

export const BuiltOnSection = () => {
  const content = {
    title: "Manage your entire end to end data stack as a unified app/service",
    description:
      "Built for real time, built for scale with batteries included infra, or BYO. Built with Rust, ClickHouse and RedPanda.",
  };
  return (
    <Section>
      <Grid className="gap-y-5">
        <HalfWidthContentContainer>
          <Heading> {content.title} </Heading>
          <Text> {content.description} </Text>
        </HalfWidthContentContainer>
        <HalfWidthContentContainer className="2xl:col-start-9 2xl:col-span-3">
          <PlaceholderImage className="aspect-square bg-muted" />
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
