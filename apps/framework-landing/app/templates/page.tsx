import { EmailSection } from "../sections/EmailSection";
import { FooterSection } from "../sections/FooterSection";
import { TemplatesSection } from "../sections/home/TemplatesSection";
import {
  Grid,
  Section,
  FullWidthContentContainer,
  HalfWidthContentContainer,
} from "design-system/components/containers";
import { Display, Heading, Text } from "design-system/typography";

export default function TemplatesPage() {
  const content = {
    title: "Templates",
    hook: {
      title: "Get started in no time",
      description:
        "Jobs, pipelines, streams, data models, tables, views, schemas, APIs, and SDKs -- no more coordinating a tangled web of individual components. With a framework-based approach, each component is aware of the bigger picture, so your data stack is easier to manage and more resilient to change.",
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
