import Link from "next/link";
import {
  FullWidthContentContainer,
  Grid,
  Section,
} from "@514labs/design-system-components/components/containers";
import { CTABar } from "../../page";
import {
  Heading,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import { TrackCtaButton } from "../../trackable-components";
import React from "react";

export const SecondaryCTASection = () => {
  const content = {
    title: "Up and running in minutes, no vendor lock-in",
    description: "Build your own data-driven experiences in minutes with Moose",
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
    <Section className="mx-auto py-24 xl:max-w-screen-xl">
      <Grid className="gap-y-5">
        <FullWidthContentContainer>
          <Heading>{content.title}</Heading>
          <Heading level={HeadingLevel.l2} className="text-muted-foreground">
            {content.description}
          </Heading>
          <CTABar>
            {content.ctas.map((cta, index) => (
              <Link key={index} href={cta.href}>
                <TrackCtaButton
                  name={content.title}
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
  );
};
