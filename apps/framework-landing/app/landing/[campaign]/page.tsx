import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "@514labs/design-system/components/containers";
import FooterSection from "../../sections/FooterSection";
import { FeatureGrid } from "../../sections/home/FeatureGrid";
import { campaignContent, defaultContent } from "./campaign-content";
import { CTASectionContent } from "../../sections/home/SecondaryCTASection";
import { Heading, HeadingLevel } from "@514labs/design-system/typography";
import { CTABar } from "@514labs/design-system/components";
import { TrackCtaButton } from "../../trackable-components";
import Link from "next/link";

export default function Page({
  params: { campaign },
}: {
  params: { campaign: string };
}) {
  const content = campaignContent?.[campaign] || defaultContent;
  return (
    <div>
      <CTASection content={content.ctaSection} />
      <Section className="mx-auto xl:max-w-screen-xl">
        <FeatureGrid features={content.features} />
      </Section>
      <FooterSection />
    </div>
  );
}

const CTASection = ({ content }: { content: CTASectionContent }) => {
  return (
    <Section className="w-full relative mx-auto xl:max-w-screen-xl pb-10">
      <Grid className="gap-y-5">
        <HalfWidthContentContainer>
          <Heading>{content.title}</Heading>
          <Heading level={HeadingLevel.l2} className="text-muted-foreground">
            {content.description}
          </Heading>
        </HalfWidthContentContainer>

        <HalfWidthContentContainer>
          <CTABar className="my-5">
            {content.ctas.map((cta, index) => (
              <Link key={index} href={cta.href}>
                <TrackCtaButton
                  name={content.title}
                  subject={cta.label}
                  variant={cta.variant as "default" | "outline"}
                >
                  {cta.label}
                </TrackCtaButton>
              </Link>
            ))}
          </CTABar>
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
