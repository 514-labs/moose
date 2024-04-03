import Link from "next/link";
import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";
import { CTABar } from "../../page";
import { Heading, Text } from "design-system/typography";
import { TrackCtaButton } from "../../trackable-components";

export const SecondaryCTASection = () => {
  const content = {
    sections: [
      {
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
      },
      {
        title: "Communities",
        description:
          "We aim to build a place for developers to get together, share feedback and gain early access to our journey.",
        ctas: [
          {
            action: "cta-join-community",
            label: "Slack",
            href: "https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg",
            variant: "default",
          },
          {
            action: "cta-join-community",
            label: "GitHub",
            href: "https://github.com/514-labs/moose",
            variant: "outline",
          },
        ],
      },
    ],
  };

  return (
    <Section>
      <Grid className="gap-y-5">
        {content.sections.map((section, index) => (
          <HalfWidthContentContainer key={index}>
            <Heading>{section.title}</Heading>
            <Text>{section.description}</Text>
            <CTABar>
              {section.ctas.map((cta, index) => (
                <Link key={index} href={cta.href}>
                  <TrackCtaButton
                    name={section.title}
                    subject={cta.label}
                    variant={cta.variant as "default" | "outline"}
                  >
                    {cta.label}
                  </TrackCtaButton>
                </Link>
              ))}
            </CTABar>
          </HalfWidthContentContainer>
        ))}
      </Grid>
    </Section>
  );
};
