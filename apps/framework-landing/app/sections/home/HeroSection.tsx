import {
  QuarterWidthContentContainer,
  HalfWidthContentContainer,
  Section,
  Grid,
} from "@/components/containers/page-containers";
import { Display, Text } from "@/components/typography/standard";
import { CTABar, CTAText, CTAButton, PlaceholderImage } from "../../page";

export const HeroSection = () => {
  const content = {
    label: "For all developers",
    tagLine: "Delightful & Insightful",
    description: "The developer framework for your data & analytics stack",
    cta: {
      action: "cta-copy",
      label: "Copy",
      text: "npx create-moose-app",
    },
  };

  return (
    <>
      <Section>
        <Grid>
          <HalfWidthContentContainer className="xl:col-start-7 md:col-span-9 xl:col-span-6">
            <div>
              <Text> {content.label} </Text>
              <Display> {content.tagLine} </Display>
              <Text> {content.description} </Text>
            </div>
            <CTABar>
              <CTAText> {content.cta.text} </CTAText>
              <CTAButton>{content.cta.label}</CTAButton>
            </CTABar>
          </HalfWidthContentContainer>
        </Grid>
      </Section>
      <Section gutterless>
        <Grid className="gap-5">
          <QuarterWidthContentContainer className="bg-muted aspect-square">
            <PlaceholderImage />
          </QuarterWidthContentContainer>
          <QuarterWidthContentContainer className="bg-muted aspect-square">
            <PlaceholderImage />
          </QuarterWidthContentContainer>
          <QuarterWidthContentContainer className="bg-muted aspect-square">
            <PlaceholderImage />
          </QuarterWidthContentContainer>
          <QuarterWidthContentContainer className="bg-muted aspect-square">
            <PlaceholderImage />
          </QuarterWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
