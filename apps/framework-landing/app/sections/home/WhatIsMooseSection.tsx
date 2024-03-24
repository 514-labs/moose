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
        <HalfWidthContentContainer className="">
          <PlaceholderImage className="aspect-square bg-muted" />
        </HalfWidthContentContainer>
        <HalfWidthContentContainer className="flex flex-col md:col-span-12 xl:col-span-6 justify-center">
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
