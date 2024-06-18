import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "@514labs/design-system-components/components/containers";
import { CTABar } from "../../page";
import { Heading, Text } from "@514labs/design-system-components/typography";
import Image from "next/image";
import { TrackCtaButton } from "../../trackable-components";
import Link from "next/link";
import React from "react";

export const WhatIsMooseSection = () => {
  const content = {
    title: "A full-stack data-engineering framework built for all devs",
    description:
      "Moose takes the decades-old best practices of frontend and backend developer frameworks, and brings them to the your data & analytics stack.",
    cta: {
      action: "cta-how-it-works-nav",
      href: "https://docs.moosejs.com/",
      label: "See how it works",
    },
  };

  return (
    <>
      <Section>
        <Grid className="gap-y-5">
          <HalfWidthContentContainer>
            <Heading> {content.title} </Heading>
            <Text> {content.description} </Text>
            <Link href={content.cta.href}>
              <CTABar>
                <TrackCtaButton name="How it works" subject={content.cta.label}>
                  {content.cta.label}
                </TrackCtaButton>
              </CTABar>
            </Link>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer className="lg:col-span-3 aspect-square bg-muted sticky md:top-24 flex items-center justify-center">
            <div className="relative w-full h-3/4">
              <Image
                priority
                className="hidden dark:block"
                src="/images/how-it-works/img-diagram-compare-dark.svg"
                fill
                alt="man in jacket"
                sizes="(max-width: 768px) 150vw, 25vw"
              />
              <Image
                priority
                className="block dark:hidden"
                src="/images/how-it-works/img-diagram-compare-light.svg"
                fill
                alt="man in jacket"
                sizes="(max-width: 768px) 150vw, 25vw"
              />
            </div>
          </HalfWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
