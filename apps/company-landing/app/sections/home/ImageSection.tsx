import {
  Section,
  Grid,
  HalfWidthContentContainer,
} from "design-system/components/containers";
import { Heading, Text } from "design-system/typography";
//import { CTABar } from "../../page";
import Image from "next/image";
//import { TrackableCodeSnippet } from "../../trackable-components";

export const ImageSection = () => {
  const content = {
    title: "If you have a computer, you're a developer",
    description:
      "We believe that data is the ultimate bridge between the arts and the sciences. Our team brings together culture, creativity, and technology to build the future of data.",
    // cta: {
    //   action: "Copy Install",
    //   label: "Copy",
    //   text: "npx create-moose-app my-moose-app",
    // },
  };
  return (
    <Section>
      <Grid className="gap-5-y">
        <HalfWidthContentContainer className="lg:col-span-3 aspect-square bg-muted sticky md:top-24">
          <div className="relative h-full">
            <Image
              priority
              src="/images/about/img_city_5.webp"
              fill
              alt="city"
              sizes=" (max-width: 768px) 150vw, 25vw"
            />
          </div>
        </HalfWidthContentContainer>
        <HalfWidthContentContainer className="lg:col-start-7">
          <Heading> {content.title} </Heading>
          <Text> {content.description} </Text>
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
