import { CTABar } from "../../page";
import {
  Section,
  Grid,
  FullWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import {
  Heading,
  Display,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import { TrackCtaButton } from "../../trackable-components";
import React from "react";
import Link from "next/link";

export const HeroSection = () => {
  const content = {
    tagLine: "Prototype & scale data-intensive apps in minutes",
    description:
      "Moose is an open source developer framework for your data & analytics stack",
    ctas: [
      {
        href: "https://docs.moosejs.com/getting-started/new-project",
        action: "cta-early-access",
        label: "Get Started",
        variant: "default",
      },
      {
        href: "https://docs.moosejs.com/",
        action: "cta-early-access",
        label: "View Docs ",
        variant: "outline",
      },
    ],
  };

  return (
    <>
      <Section className="max-w-5xl mx-auto flex flex-col items-center px-5 my-16 sm:my-32">
        <Grid>
          <FullWidthContentContainer className="pt-0 items-center px-20 flex flex-col gap-5">
            <div>
              {/* <Heading> {content.tagLine} </Heading> */}
              <Display className="my-0 text-center">{content.tagLine} </Display>

              <Heading
                level={HeadingLevel.l3}
                className="text-muted-foreground text-center"
              >
                {" "}
                {content.description}{" "}
              </Heading>
            </div>
            <CTABar className="mb-10 align-center justify-center">
              {content.ctas.map((cta, index) => (
                <Link key={index} href={cta.href}>
                  <TrackCtaButton
                    name={`Hero CTA ${cta.label}`}
                    subject={content.tagLine}
                    targetUrl={cta.href}
                    variant={cta.variant as "default" | "outline"}
                  >
                    {cta.label}
                  </TrackCtaButton>
                </Link>
              ))}
            </CTABar>
          </FullWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
