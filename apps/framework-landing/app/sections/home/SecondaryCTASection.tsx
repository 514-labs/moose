import {
  HalfWidthContentContainer,
  Section,
  Grid,
} from "@/components/containers/page-containers";
import { Heading, Text } from "@/components/typography/standard";
import { CTABar, CTAButton } from "../../page";

export const SecondaryCTASection = () => {
  const content = {
    sections: [
      {
        title: "Hosting",
        description:
          "Don't want to manage hosting for your Moose application? Check out Fiveonefour.",
        ctas: [
          {
            action: "cta-early-access",
            label: "Get early access",
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
            href: "#",
          },
          {
            action: "cta-join-community",
            label: "GitHub",
            href: "#",
          },
        ],
      },
    ],
  };

  return (
    <Section>
      <Grid className="gap-y-5">
        <HalfWidthContentContainer>
          <Heading> {content.sections[0].title} </Heading>
          <Text> {content.sections[0].description} </Text>
          <CTABar>
            <CTAButton> {content.sections[0].ctas[0].label} </CTAButton>
          </CTABar>
        </HalfWidthContentContainer>
        <HalfWidthContentContainer>
          <Heading> {content.sections[1].title} </Heading>
          <Text> {content.sections[1].description} </Text>
          <CTABar>
            <CTAButton>{content.sections[1].ctas[0].label} </CTAButton>
            <CTAButton variant={"outline"}>
              {content.sections[1].ctas[1].label}
            </CTAButton>
          </CTABar>
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
