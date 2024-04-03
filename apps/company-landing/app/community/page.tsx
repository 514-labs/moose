import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";
import { Display, Heading, Text } from "design-system/typography";
import { CTABar } from "../../components.tsx/CTAs";
import FooterSection from "../sections/FooterSection";
import { EmailSection } from "../sections/EmailSection";
import Link from "next/link";
import { TrackCTAButton } from "../trackable-components";

const content = {
  title: "Community",
  description: {
    headline: "Join our dev communities",
    body: "We aim to build a place for developers to get together, share feedback and gain early access to our journey.",
  },
  reasons: [
    {
      title: "Feedback",
      description:
        "We're in the early stages, and your feedback is like gold to us. We want to hear your thoughts, and suggestions to make our product great.",
    },
    {
      title: "Insights",
      description:
        "We're eager to learn from your interactions with our product. Your insights help us uncover new possibilities and address any issues you encounter.",
    },
    {
      title: "Early Access",
      description:
        "Be among the first to try our product. Early access means you're not just a user; you're a co-creator. Together, we're building something amazing.",
    },
  ],

  communities: [
    {
      title: "Slack",
      description:
        "A place for open product discussions, private groups, sharing feedback and getting early access.",
      href: "https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg",
    },
    {
      title: "Github",
      description:
        "Contribute to our open-source projects, report issues, and suggest new features.",
      href: "https://github.com/514-labs/moose",
    },
  ],
};

export default function Community() {
  return (
    <>
      <Section>
        <Display>Community</Display>
      </Section>
      <Section>
        <Grid>
          <HalfWidthContentContainer>
            <Heading>{content.description.headline}</Heading>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer>
            <Text>{content.description.body}</Text>
          </HalfWidthContentContainer>
        </Grid>
      </Section>
      <Section>
        {content.reasons.map((reason, i) => (
          <Grid key={i}>
            <HalfWidthContentContainer>
              <Text>0{i + 1}</Text>
            </HalfWidthContentContainer>
            <HalfWidthContentContainer>
              <Heading>{reason.title}</Heading>
              <Text>{reason.description}</Text>
            </HalfWidthContentContainer>
          </Grid>
        ))}
      </Section>
      <Section>
        <Heading>Pick your community</Heading>
      </Section>
      <Section>
        {/* {content.communities.map((community, i) => (
            <HalfWidthContentContainer key={i}>
              <Heading>{community.title}</Heading>
              <Text>{community.description}</Text>
              <CTABar>
                <Link href={community.href}>
                  <CTAButton variant={"outline"}>Join</CTAButton>
                </Link>
              </CTABar>
            </HalfWidthContentContainer>
          ))} */}
        {content.communities.map((community, i) => (
          <Grid key={i}>
            <HalfWidthContentContainer>
              <Text>0{i + 1}</Text>
            </HalfWidthContentContainer>
            <HalfWidthContentContainer className="py-5">
              <Heading>{community.title}</Heading>
              <Text>{community.description}</Text>
              <CTABar>
                <Link href={community.href}>
                  <TrackCTAButton
                    name={"Join Community"}
                    subject={community.title}
                    variant={"outline"}
                  >
                    Join
                  </TrackCTAButton>
                </Link>
              </CTABar>
            </HalfWidthContentContainer>
          </Grid>
        ))}
      </Section>
      <FooterSection />
      <EmailSection />
    </>
  );
}
