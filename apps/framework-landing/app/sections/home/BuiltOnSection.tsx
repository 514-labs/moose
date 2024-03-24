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
        <HalfWidthContentContainer className="flex flex-col justify-center">
          <Heading> {content.title} </Heading>
          <Text> {content.description} </Text>
        </HalfWidthContentContainer>
        <HalfWidthContentContainer>
          <PlaceholderImage className="aspect-square bg-muted" />
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
