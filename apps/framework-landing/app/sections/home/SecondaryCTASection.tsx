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
    title: "Hosting",
    description:
      "Don't want to manage hosting for your Moose application? Check out Fiveonefour.",
    ctas: [
      {
        href: "https://fiveonefour.com",
        action: "cta-early-access",
        label: "Get early access",
        variant: "default",
      },
    ],
  };

  return (
    <Section>
      <Grid className="gap-y-5">
        <FullWidthContentContainer>
          <Heading>{content.title}</Heading>
          <Text>{content.description}</Text>
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
