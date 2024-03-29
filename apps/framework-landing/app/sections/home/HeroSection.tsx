import { CodeSnippet as AnimatedCodeSnipper } from "design-system/typography/animated";
import { CTABar, PlaceholderImage } from "../../page";
import {
  Section,
  HalfWidthContentContainer,
  QuarterWidthContentContainer,
  Grid,
} from "design-system/components/containers";
import { Display, Text } from "design-system/typography";
import Image from "next/image";

export const HeroSection = () => {
  const content = {
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
          <QuarterWidthContentContainer className="bg-muted aspect-square relative">
            <Image
              priority
              src="/images/hero/mjs_img_4.webp"
              fill
              alt="moose"
              sizes=" (max-width: 768px) 150vw, 25vw"
            />
          </QuarterWidthContentContainer>
          <QuarterWidthContentContainer className="bg-muted aspect-square relative">
            <Image
              priority
              src="/images/hero/mjs_img_2.webp"
              fill
              alt="girl"
              sizes=" (max-width: 768px) 150vw, 25vw"
            />
          </QuarterWidthContentContainer>
          <QuarterWidthContentContainer className="bg-muted aspect-square relative">
            <Image
              priority
              src="/images/hero/mjs_img_3.webp"
              fill
              alt="laptop on table"
              sizes=" (max-width: 768px) 150vw, 25vw"
            />
          </QuarterWidthContentContainer>
          <QuarterWidthContentContainer className="bg-muted aspect-square relative">
            <Image
              priority
              src="/images/hero/mjs_img_6.webp"
              fill
              alt="man in jacket"
              sizes=" (max-width: 768px) 150vw, 25vw"
            />
          </QuarterWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
