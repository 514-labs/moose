import { Heading, Text } from "design-system/typography";
import { PlaceholderImage } from "../../page";
import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";
import Image from "next/image";

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
        <HalfWidthContentContainer className="2xl:col-start-10 2xl:col-span-3 aspect-square bg-muted relative">
          <Image
            priority
            src="/images/built-on/mjs-img_logo_logo.gif"
            fill
            alt="man in jacket"
            sizes=" (max-width: 768px) 150vw, 25vw"
          />
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
