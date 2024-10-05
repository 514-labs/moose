import {
  Section,
  Grid,
  HalfWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import { Heading, Text } from "@514labs/design-system-components/typography";
import { CTABar } from "../../page";
import Image from "next/image";
import { TrackableCodeSnippet } from "@514labs/design-system-components/trackable-components";

export const GetMooseCTASection = () => {
  const content = {
    title: "Try Moose",
    description:
      "Easily build data-intensive applications with Moose. Get started with a single command.",
    cta: {
      action: "Copy Install",
      label: "Copy",
      text: "npx create-moose-app my-moose-app",
    },
  };
  return (
    <Section>
      <Grid className="gap-5-y">
        <HalfWidthContentContainer className="lg:col-span-3 aspect-square bg-muted sticky md:top-24 flex items-center justify-center">
          <div className="relative w-full h-full">
            <Image
              priority
              src="/images/get-moose/mjs_img_7.webp"
              fill
              alt="man in jacket"
              sizes=" (max-width: 768px) 150vw, 25vw"
            />
          </div>
        </HalfWidthContentContainer>
        <HalfWidthContentContainer className="lg:col-start-7">
          <Heading> {content.title} </Heading>
          <Text> {content.description} </Text>
          <CTABar>
            <TrackableCodeSnippet
              name={content.cta.action}
              subject={content.cta.text}
            >
              {content.cta.text}
            </TrackableCodeSnippet>
          </CTABar>
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
