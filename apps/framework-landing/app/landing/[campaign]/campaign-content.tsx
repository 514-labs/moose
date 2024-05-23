import { featureContent } from "../../sections/home/FeaturesSection";
import { CTASectionContent } from "../../sections/home/SecondaryCTASection";

interface CampaignContent {
  ctaSection: CTASectionContent;
  features: { title: string; description: string }[];
}

interface ContentMap {
  [key: string]: CampaignContent;
}

const defaultCTASectionContent: CTASectionContent = {
  title: "Up and running in minutes, no vendor, no lock-in",
  description: "Build your own data-driven experiences in minutes with Moose",
  ctas: [
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
  ],
};

export const campaignContent: ContentMap = {
  ["mixpanel-alternative"]: {
    ctaSection: defaultCTASectionContent,
    features: featureContent.features,
  },
};

export const defaultContent: CampaignContent = {
  ctaSection: defaultCTASectionContent,
  features: featureContent.features,
};
