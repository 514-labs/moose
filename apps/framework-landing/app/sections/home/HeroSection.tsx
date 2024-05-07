import { CTABar } from "../../page";
import {
  Section,
  HalfWidthContentContainer,
  QuarterWidthContentContainer,
  Grid,
} from "@514labs/design-system/components/containers";
import { Heading, HeadingLevel} from "@514labs/design-system/typography";
import Image from "next/image";
import {
  TrackCtaButton,
  TrackableCodeSnippet,
} from "../../trackable-components";

export const HeroSection = () => {
  const content = {
    tagLine: "A developer framework for your data & analytics stack",
    description: "Build your own data driven experiences with in minutes",
    primaryCta: {
      action: "cta-copy",
      label: "Copy",
      text: "Get Started",
    },
    secondaryCta: {
      action: "cta-docs",
      label: "Docs",
      text: "View Docs",
    },
  };

  return (
    <>
      <Section className="mt-12 lg:mt-12 2xl:mt-24 px-5">
        <Grid>
          <FullWidthContentContainer className="">
            <div>
              <Heading> {content.tagLine} </Heading>
              <Heading
                level={HeadingLevel.l2}
                className="text-muted-foreground"
              >
                {" "}
                {content.description}{" "}
              </Heading>
            </div>
            <CTABar className="mb-5">
              <TrackCtaButton
                name="Get Started"
                subject={content.primaryCta.text}
              >
                {content.primaryCta.text}
              </TrackCtaButton>
              <TrackCtaButton
                name="View Docs"
                subject={content.secondaryCta.text}
                variant={"outline"}
              >
                {content.secondaryCta.text}
              </TrackCtaButton>
            </CTABar>
          </FullWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
