import {
  Grid,
  HalfWidthContentContainer,
  Section,
  ThirdWidthContentContainer,
  FullWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import {
  // Display,
  Heading,
  Text,
  SmallText,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
// import { CTABar } from "@514labs/design-system-components/components";
import FooterSection from "../sections/FooterSection";
// import Link from "next/link";
// import { TrackCtaButton } from "../trackable-components";
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import {
  // Box,
  // Network,
  // Share2,
  // Terminal,
  // Code2,
  // Server,
  Github,
  Slack,
  MessageCircle,
  TerminalIcon,
  TrendingUp,
} from "lucide-react";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// import { cn } from "@514labs/design-system-components/lib/utils";
// import { LooseMooseSection } from "../sections/home/LooseMooseSection";

// const content = {
//   title: "Community",
//   description: {
//     headline: "Join our dev communities",
//     body: "We aim to build a place for developers to get together, share feedback and gain early access to our journey.",
//   },
//   reasons: [
//     {
//       title: "Feedback",
//       description:
//         "We're in the early stages, and your feedback is like gold to us. We want to hear your thoughts, and suggestions to make our product great.",
//     },
//     {
//       title: "Insights",
//       description:
//         "We're eager to learn from your interactions with our product. Your insights help us uncover new possibilities and address any issues you encounter.",
//     },
//     {
//       title: "Early Access",
//       description:
//         "Be among the first to try our product. Early access means you're not just a user; you're a co-creator. Together, we're building something amazing.",
//     },
//   ],

//   communities: [
//     {
//       title: "Slack",
//       description:
//         "A place for open product discussions, private groups, sharing feedback and getting early access.",
//       href: "https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg",
//     },
//     {
//       title: "Github",
//       description:
//         "Contribute to our open-source projects, report issues, and suggest new features.",
//       href: "https://github.com/514-labs/moose",
//     },
//   ],
// };

const HeroSection = ({ className }: { className?: string }) => {
  const content = {
    title: `Join Our Dev Communities`,
    description:
      "We believe that data is the ultimate bridge between the arts and the sciences. Our team brings together culture, creativity, and technology to build the future of data.",
  };

  return (
    <section className={cn(className)}>
      <div className="text-4xl sm:text-5xl md:text-6xl lg:text-6xl 2xl:text-7xl text-center z-10">
        {content.title}
      </div>
      <div className="text-2xl 2xl:text-4xl text-center text-muted-foreground py-5 z-10">
        {content.description}
      </div>
    </section>
  );
};

const PickSection = ({ className }: { className?: string }) => {
  return <section className={className}>Pick your community</section>;
};

export const SkillSection = () => {
  const content = {
    skills: [
      {
        title: "Feedback",
        description:
          "We're in the early stages, and your feedback is like gold to us. We want to hear your thoughts, and suggestions to make our product great.",
        icon: <MessageCircle strokeWidth={1} />,
      },
      {
        title: "Insights",
        description:
          "We're eager to learn from your interactions with our product. Your insights help us uncover new possibilities and address any issues you encounter",
        icon: <TrendingUp strokeWidth={1} />,
      },
      {
        title: "Early Access",
        description:
          "Be among the first to try our product. Early access means you're not just a user; you're a co-creator. Together, we're building something amazing.",
        icon: <TerminalIcon strokeWidth={1} />,
      },
    ],
  };

  return (
    <>
      <Section className="mx-auto max-w-5xl">
        <Grid className="mb-0 2xl:mb-20">
          <FullWidthContentContainer>
            <Heading
              level={HeadingLevel.l1}
              className="max-w-5xl justify-center align-center text-center sm:text-5xl"
            >
              {content.title}
            </Heading>
            <Heading
              level={HeadingLevel.l2}
              className="max-w-5xl justify-center align-center text-center text-muted-foreground"
            >
              {content.subtitle}
            </Heading>
          </FullWidthContentContainer>
        </Grid>
        <Grid className="gap-y-10">
          {content.skills.map((feature, index) => {
            return (
              <ThirdWidthContentContainer
                key={index}
                className="flex flex-col gap-5 border p-5 rounded-3xl"
              >
                {feature.icon}
                <Text className="my-0">{feature.title}</Text>
                <SmallText className="my-0 text-muted-foreground text-[20px]">
                  {feature.description}
                </SmallText>
              </ThirdWidthContentContainer>
            );
          })}
        </Grid>
      </Section>
    </>
  );
};

export const SlackSection = () => {
  const content = {
    slack: [
      {
        title: "Slack",
        description:
          "A place for open product discussions, private groups, sharing feedback and getting access.",
        icon: <Slack strokeWidth={1} />,
      },
      {
        title: "Github",
        description:
          "Contribute to our open-source projects, report issues, and suggest new features.",
        icon: <Github strokeWidth={1} />,
      },
    ],
  };

  return (
    <>
      <Section className="mx-auto max-w-5xl">
        <Grid className="mb-0">
          <FullWidthContentContainer>
            <Heading
              level={HeadingLevel.l1}
              className="max-w-5xl justify-center align-center text-center sm:text-5xl"
            >
              {content.title}
            </Heading>
            <Heading
              level={HeadingLevel.l2}
              className="max-w-5xl justify-center align-center text-center text-muted-foreground"
            >
              {content.subtitle}
            </Heading>
          </FullWidthContentContainer>
        </Grid>
        <Grid className="gap-y-10">
          {content.slack.map((feature, index) => {
            return (
              <HalfWidthContentContainer
                key={index}
                className="flex flex-col gap-5 border p-5 rounded-3xl"
              >
                {feature.icon}
                <Text className="my-0">{feature.title}</Text>
                <SmallText className="my-0 text-muted-foreground text-[20px]">
                  {feature.description}
                </SmallText>
              </HalfWidthContentContainer>
            );
          })}
        </Grid>
      </Section>
    </>
  );
};

export default function Community() {
  return (
    <>
      {/* <Section className="px-6 w-full relative max-w-5xl mx-auto px-5 text-center ">
        <Display>Community</Display>
      </Section> */}
      <HeroSection className="max-w-5xl mx-auto px-10 py-5 p-20 lg:py-0 flex flex-col items-center my-16 sm:my-40" />
      <SkillSection />
      <PickSection className="max-w-5xl mx-auto p-10 text-center text-3xl sm:text-5xl pb-0 mb-0" />
      <SlackSection />
      <FooterSection />

      {/*       
      <Section className="px-6 w-full relative">
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
      <Section className="px-6 w-full relative max-w-5xl mx-auto px-5 text-center">
        <Heading>Pick your community</Heading>
      </Section>
      <Section className="px-6 w-full relative mx-auto xl:max-w-screen-xl">
        {content.communities.map((community, i) => (
          <Grid key={i} className="py-5 md:py-0">
            <HalfWidthContentContainer>
              <Text className="p-0 m-0">0{i + 1}</Text>
            </HalfWidthContentContainer>
            <HalfWidthContentContainer className="py-5">
              <Heading>{community.title}</Heading>
              <Text>{community.description}</Text>
              <CTABar>
                <Link href={community.href}>
                  <TrackCtaButton
                    name={community.title} // Add any necessary props for tracking
                    subject="Join Community" // Add any necessary props for tracking
                    variant={"outline"}
                  >
                    Join
                  </TrackCtaButton>
                </Link>
              </CTABar>
            </HalfWidthContentContainer>
          </Grid>
        ))}
      </Section> */}
      {/* <EmailSection /> */}
      {/* <LooseMooseSection /> */}
    </>
  );
}
