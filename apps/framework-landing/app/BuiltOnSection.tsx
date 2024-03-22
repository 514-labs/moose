import {
  HalfWidthContentContainer,
  Section,
  Grid,
} from "@/components/containers/page-containers";
import { Heading, Text } from "@/components/typography/standard";
import { PlaceholderImage } from "./page";

export const BuiltOnSection = () => {
  const content = {
    title: "Manage your entire end to end data stack as a unified app/service",
    description:
      "Built for real time, built for scale with batteries included infra, or BYO. Built with Rust, Clickhouse and Red Panda.",
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
