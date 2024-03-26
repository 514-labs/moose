import {
  Section,
  HalfWidthContentContainer,
  Grid,
} from "design-system/components/containers";

import { PlaceholderImage, CTABar, CTAButton } from "../../page";
import { Heading, Text } from "design-system/typography";

export const WhatIsMooseSection = () => {
  const content = {
    title: "A full-stack data-engineering framework built for all devs",
    description:
      "Moose takes the decades-old best practices of frontend and backend developer frameworks, and brings them to the your data & analytics stack.",
    cta: {
      action: "cta-how-it-works-nav",
      label: "See how it works",
    },
  };

  return (
    <Section>
      <Grid className="gap-y-5">
        <HalfWidthContentContainer className="2xl:col-span-3">
          <PlaceholderImage className="aspect-square bg-muted" />
        </HalfWidthContentContainer>
        <HalfWidthContentContainer className="md:col-span-12 xl:col-span-6 2xl:col-start-7">
          <Heading> {content.title} </Heading>
          <Text> {content.description} </Text>
          <CTABar>
            <CTAButton> {content.cta.label} </CTAButton>
          </CTABar>
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
