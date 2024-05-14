import { CTABar } from "../../page";
import {
  Section,
  Grid,
  FullWidthContentContainer,
} from "@514labs/design-system/components/containers";
import {
  Heading,
  Display,
  HeadingLevel,
} from "@514labs/design-system/typography";
import { TrackCtaButton } from "../../trackable-components";
import React from "react";
import Link from "next/link";

export const HeroSection = () => {
  const content = {
    tagLine: "Build your own data products in minutes",
    description:
      "An open source developer framework for your data & analytics stack",
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
        label: "Learn More",
        variant: "outline",
      },
    ],
  };

  return (
    <>
      <Section className="w-full relative mx-auto xl:max-w-screen-xl pb-10">
        <Grid>
          <FullWidthContentContainer className="pt-0">
            <div>
              {/* <Heading> {content.tagLine} </Heading> */}
              <Display className="my-0">{content.tagLine} </Display>

              <Heading
                level={HeadingLevel.l2}
                className="text-muted-foreground"
              >
                {" "}
                {content.description}{" "}
              </Heading>
            </div>
            <CTABar className="mb-10">
              {content.ctas.map((cta, index) => (
                <Link key={index} href={cta.href}>
                  <TrackCtaButton
                    name={cta.label}
                    subject={cta.label}
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
