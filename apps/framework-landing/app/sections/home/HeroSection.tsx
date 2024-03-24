import { CodeSnippet as AnimatedCodeSnipper } from "design-system/typography/animated";
import { CTABar, PlaceholderImage } from "../../page";
import {
  Section,
  HalfWidthContentContainer,
  QuarterWidthContentContainer,
  Grid,
} from "design-system/components/containers";
import { Display, Text } from "design-system/typography";

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
              <AnimatedCodeSnipper> {content.cta.text} </AnimatedCodeSnipper>
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
