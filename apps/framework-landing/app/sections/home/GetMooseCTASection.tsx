import {
  Section,
  Grid,
  HalfWidthContentContainer,
} from "design-system/components/containers";
import { Heading, Text } from "design-system/typography";
import { CodeSnippet } from "design-system/typography/animated";
import { PlaceholderImage, CTABar } from "../../page";

export const GetMooseCTASection = () => {
  const content = {
    title: "Try moose",
    description:
      "Moose takes the decades-old best practices of frontend and backend developer frameworks, and brings them to the your data & analytics stack.",
    cta: {
      action: "cta-early-access",
      label: "Copy",
      text: "npx create-moose-app",
    },
  };
  return (
    <Section>
      <Grid className="gap-5-y">
        <HalfWidthContentContainer className="2xl:col-span-3">
          <PlaceholderImage className="aspect-square bg-muted" />
        </HalfWidthContentContainer>
        <HalfWidthContentContainer className="2xl:col-start-7">
          <Heading> {content.title} </Heading>
          <Text> {content.description} </Text>
          <CTABar>
            <CodeSnippet> {content.cta.text} </CodeSnippet>
          </CTABar>
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
