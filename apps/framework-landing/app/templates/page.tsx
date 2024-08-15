import React from "react";
import { FooterSection } from "../sections/FooterSection";
import { LooseMooseSection } from "../sections/home/LooseMooseSection";
import { TemplatesSection } from "../sections/home/TemplatesSection";
import {
  Grid,
  Section,
  FullWidthContentContainer,
  HalfWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import {
  Display,
  Heading,
  Text,
} from "@514labs/design-system-components/typography";

export default function TemplatesPage() {
  const content = {
    title: "Templates",
    hook: {
      title: "Kickstart your next project",
      description:
        "Explore our collection of example data intensive applications, built on Moose. These templates are composable and fully customizable to meet your specific needs.",
    },
  };

  return (
    <>
      <Section className="w-full relative mx-auto max-w-5xl">
        <FullWidthContentContainer>
          <Display>{content.title}</Display>
        </FullWidthContentContainer>
      </Section>
      <Section className="w-full relative mx-auto max-w-5xl">
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
      {/* <LooseMooseSection /> */}
    </>
  );
}
