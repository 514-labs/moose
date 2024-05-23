import { featureContent } from "../../sections/home/FeaturesSection";
import { CTASectionContent } from "../../sections/home/SecondaryCTASection";

interface CampaignContent {
  ctaSection: CTASectionContent;
  features: { title: string; description: string }[];
}

interface ContentMap {
  [key: string]: CampaignContent;
}

const defaultCtas = [
  {
    href: "https://fiveonefour.typeform.com/signup",
    action: "cta-join-waitlist",
    label: "Join Waitlist",
    variant: "default",
  },
  {
    href: "https://docs.moosejs.com/",
    action: "cta-early-access",
    label: "View Docs ",
    variant: "outline",
  },
];

const defaultCTASectionContent: CTASectionContent = {
  title: "Up and running in minutes, no vendor, no lock-in",
  description: "Build your own data-driven experiences in minutes with Moose",
  ctas: defaultCtas,
};

export const campaignContent: ContentMap = {
  ["mixpanel-alternative"]: {
    ctaSection: {
      title: "Build an Open Source Mixpanel Alternative",
      description:
        "Use Moose templates to own a Mixpanel alternative in minutes",
      ctas: defaultCtas,
    },
    features: featureContent.features,
  },
};

export const defaultContent: CampaignContent = {
  ctaSection: defaultCTASectionContent,
  features: featureContent.features,
};
