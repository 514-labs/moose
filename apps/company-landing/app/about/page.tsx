import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "@514labs/design-system/components/containers";
import { Display, Heading, Text } from "@514labs/design-system/typography";
import FooterSection from "../sections/FooterSection";
// import { EmailSection } from "../sections/EmailSection";
import { ImageSection } from "../sections/home/ImageSection";
//import { TrackCtaButton } from "../trackable-components";

const content = {
  title: "About",
  description: {
    headline: "We’re at the intersection of art & science",
    body: "We believe that data is the ultimate bridge between the arts and the sciences. Our team brings together culture, creativity, and technology to build the future of data.",
  },
  reasons: [
    {
      title: "Celebrate builders",
      description:
        "We are builders. We build for builders. The ones who create, the ones who dream, the ones who do.",
    },
    {
      title: "Be flexible",
      description:
        "We are builders. We build for builders. The ones who create, the ones who dream, the ones who do.",
    },
    {
      title: "Deliver value",
      description:
        "We are focused on creating value for our team, our community, our customers, and our investors.",
    },
  ],

  investors: [
    {
      company: "Dimension Capital",
      title: "Nan Li",
      description: "Managing Director",
    },
    {
      company: "Flybridge",
      title: "Chip Hazard",
      description: "General Partner",
    },
    {
      company: "Ridge Ventures",
      title: "Akriti Dokania",
      description: "Partner",
    },
    {
      company: "Clever",
      title: "Tyler Bosmeny",
      description: "CEO",
    },
    {
      company: "Levy",
      title: "Adam Spector",
      description: "Founder",
    },
    {
      company: "Chime",
      title: "Zachary Smith",
      description: "SVP Product",
    },
    {
      title: "Pardhu Gunnam",
      company: "Metaphor",
      description: "CEO",
    },
    {
      title: "Hareesh Pottamsetty",
      company: "Google Play",
      description: "Monetization Lead",
    },
    {
      title: "Savin Goyal",
      company: "Outerbounds",
      description: "CEO",
    },
  ],
};

export default function About() {
  return (
    <>
      <Section className="px-6 w-full relative mx-auto xl:max-w-screen-xl">
        <Display>About</Display>
      </Section>
      <Section className="px-6 w-full relative mx-auto xl:max-w-screen-xl">
        <Grid>
          <HalfWidthContentContainer>
            <Heading>{content.description.headline}</Heading>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer>
            <Text>{content.description.body}</Text>
          </HalfWidthContentContainer>
        </Grid>
      </Section>
      <Section className="px-6 w-full relative mx-auto xl:max-w-screen-xl">
        {content.reasons.map((reason, i) => (
          <Grid key={i} className="py-5 md:py-0">
            <HalfWidthContentContainer>
              <Text className="p-0 m-0">0{i + 1}</Text>
            </HalfWidthContentContainer>
            <HalfWidthContentContainer>
              <Heading>{reason.title}</Heading>
              <Text>{reason.description}</Text>
            </HalfWidthContentContainer>
          </Grid>
        ))}
      </Section>
      <Section className="px-6 w-full relative mx-auto xl:max-w-screen-xl">
        <Heading>Backed by the bold</Heading>
      </Section>
      <Section className="px-6 w-full relative mx-auto xl:max-w-screen-xl">
        {content.investors.map((about, i) => (
          <Grid key={i} className="py-5 md:py-0">
            <HalfWidthContentContainer>
              <Text className="p-0 m-0">{about.company}</Text>
            </HalfWidthContentContainer>
            <HalfWidthContentContainer>
              <Heading>{about.title}</Heading>
              <Text>{about.description}</Text>
            </HalfWidthContentContainer>
          </Grid>
        ))}
        <ImageSection />
        <FooterSection />
      </Section>

      {/* <EmailSection /> */}
    </>
  );
}
