import React from "react";
import { FooterSection } from "../sections/FooterSection";
import { LooseMooseSection } from "../sections/home/LooseMooseSection";
import { TemplatesSection } from "../sections/home/TemplatesSection";
import {
  Grid,
  Section,
  FullWidthContentContainer,
  HalfWidthContentContainer,
} from "@514labs/design-system/components/containers";
import { Display, Heading, Text } from "@514labs/design-system/typography";

export default function TemplatesPage() {
  const content = {
    title: "Templates",
    hook: {
      title: "Get started in no time",
      description:
        "Explore plug-and-play applications built with MooseJS, designed for immediate deployment, and fully customizable to meet your specific needs. Adapt the data models, flows, and insights effortlessly— all while using the programming languages and practices you prefer.",
    },
  };

  return (
    <>
      <Section className="w-full relative mx-auto xl:max-w-screen-xl">
        <FullWidthContentContainer>
          <Display>{content.title}</Display>
        </FullWidthContentContainer>
      </Section>
      <Section className="w-full relative mx-auto xl:max-w-screen-xl">
        <Grid>
          <HalfWidthContentContainer>
            <Heading>{content.hook.title}</Heading>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer>
            <Text>{content.hook.description}</Text>
          </HalfWidthContentContainer>
        </Grid>
      </Section>
      <TemplatesSection />
      <FooterSection />
      {/* <EmailSection /> */}
      <LooseMooseSection />
    </>
  );
}
