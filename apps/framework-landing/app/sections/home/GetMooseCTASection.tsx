import {
  HalfWidthContentContainer,
  Section,
  Grid,
} from "@/components/containers/page-containers";
import { CodeSnippet, Display, Text } from "@/components/typography/standard";
import { PlaceholderImage, CTABar, CTAText, CTAButton } from "../../page";

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
        <HalfWidthContentContainer className="">
          <PlaceholderImage className="aspect-square bg-muted" />
        </HalfWidthContentContainer>
        <HalfWidthContentContainer className="flex flex-col justify-center">
          <Display> {content.title} </Display>
          <Text> {content.description} </Text>
          <CTABar>
            <CodeSnippet> {content.cta.text} </CodeSnippet>
          </CTABar>
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
