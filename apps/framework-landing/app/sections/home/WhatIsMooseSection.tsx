import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";
import { CTABar } from "../../page";
import { Heading, Text } from "design-system/typography";
import Image from "next/image";
import { TrackCtaButton } from "../../trackable-components";
import Link from "next/link";

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
          <HalfWidthContentContainer className="2xl:col-span-3 aspect-square bg-muted flex flex-col item-center justify-center">
            <div className="relative h-3/4">
              <Image
                priority
                className="hidden dark:block"
                src="/images/how-it-works/img-diagram-compare-dark.svg"
                fill
                alt="man in jacket"
                sizes=" (max-width: 768px) 150vw, 25vw"
              />
              <Image
                priority
                className="block dark:hidden"
                src="/images/how-it-works/img-diagram-compare-light.svg"
                fill
                alt="man in jacket"
                sizes=" (max-width: 768px) 150vw, 25vw"
              />
            </div>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer className=" 2xl:col-start-7">
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
        </Grid>
      </Section>
    </>
  );
};
