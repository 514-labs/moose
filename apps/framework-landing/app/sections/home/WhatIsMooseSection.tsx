import {
  FullWidthContentContainer,
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";
import { CTABar } from "../../page";
import { Display, Heading, Text } from "design-system/typography";
import Image from "next/image";
import { TrackCtaButton } from "../../trackable-components";

export const WhatIsMooseSection = () => {
  const content = {
    section: "How it works?",
    title: "A full-stack data-engineering framework built for all devs",
    description:
      "Moose takes the decades-old best practices of frontend and backend developer frameworks, and brings them to the your data & analytics stack.",
    cta: {
      action: "cta-how-it-works-nav",
      label: "See how it works",
    },
  };

  return (
    <>
      <Section>
        <Grid>
          <FullWidthContentContainer>
            <Display> {content.section} </Display>
          </FullWidthContentContainer>
        </Grid>
      </Section>

      <Section>
        <Grid className="gap-y-5">
          <HalfWidthContentContainer className="2xl:col-span-3 aspect-square bg-muted flex flex-col item-center justify-center">
            <div className="relative h-1/3">
              <Image
                priority
                src="/images/how-it-works/mjs-img-scaffold.svg"
                fill
                alt="man in jacket"
                sizes=" (max-width: 768px) 150vw, 25vw"
              />
            </div>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer className=" 2xl:col-start-7">
            <Heading> {content.title} </Heading>
            <Text> {content.description} </Text>
            <CTABar>
              <TrackCtaButton name="How it works" subject={content.cta.label}>
                {content.cta.label}
              </TrackCtaButton>
            </CTABar>
          </HalfWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
