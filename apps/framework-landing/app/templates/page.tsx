import { EmailSection } from "../sections/EmailSection";
import { FooterSection } from "../sections/FooterSection";
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
      <Section>
        <FullWidthContentContainer>
          <Display>{content.title}</Display>
        </FullWidthContentContainer>
      </Section>
      <Section>
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
      <EmailSection />
    </>
  );
}
