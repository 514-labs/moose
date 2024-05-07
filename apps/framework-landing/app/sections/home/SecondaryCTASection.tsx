import Link from "next/link";
import {
  FullWidthContentContainer,
  Grid,
  Section,
} from "@514labs/design-system/components/containers";
import { CTABar } from "../../page";
import { Heading, Text } from "@514labs/design-system/typography";
import { TrackCtaButton } from "../../trackable-components";

export const SecondaryCTASection = () => {
  const content = {
    title: "Up and running in minutes, no vendor, no lock-in",
    description: "Get your data stack up and running in minutes with Moose",
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
    <Section>
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
