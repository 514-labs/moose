import {
  FullWidthContentContainer,
  HalfWidthContentContainer,
  Section,
} from "@514labs/design-system-components/components/containers";

import { Text } from "@514labs/design-system-components/typography";

import { cn } from "@514labs/design-system-components/utils";
import { ThemeToggle } from "@514labs/design-system-components/components";
import { TrackLink } from "@514labs/design-system-components/trackable-components";

export const FooterSection = () => {
  return (
    <Section className="mx-auto max-w-5xl md:my-0">
      <HalfWidthContentContainer>
        <FooterContent />
      </HalfWidthContentContainer>
    </Section>
  );
};

export const FooterNavItem = ({
  item,
  children,
  className,
  textClassName,
}: {
  item: { name: string; href: string };
  children: string;
  className?: string;
  textClassName?: string;
}) => {
  return (
    <TrackLink
      name="Footer Nav"
      subject={item.name}
      href={item.href}
      className={cn("text-foreground flex flex-row justify-start", className)}
    >
      <Text className={cn("mb-0", textClassName)}> {children} </Text>
    </TrackLink>
  );
};

const footerContent = {
  products: [
    {
      name: "Moose",
      href: "https://getmoose.dev",
    },
    {
      name: "Boreal",
      href: "https://boreal.cloud",
    },
  ],
  resources: [
    {
      name: "Community",
      href: "/community",
    },
    {
      name: "Docs",
      href: "https://docs.getmoose.dev",
    },
    {
      name: "Get Started",
      href: "https://docs.getmoose.dev/quickstart",
    },
    {
      name: "Tutorials",
      href: "https://docs.getmoose.dev/learn-moose",
    },
  ],
  company: [
    {
      name: "Careers",
      href: "https://fiveonefour.com/careers",
    },
    {
      name: "About",
      href: "https://fiveonefour.com/about",
    },
    {
      name: "Blog",
      href: "https://fiveonefour.com/blog",
    },
    {
      name: "Partner Login",
      href: "https://www.fiveonefour.com/sign-in?redirect_url=https%3A%2F%2Fwww.fiveonefour.com%2Fresources",
    },
  ],
  social: [
    {
      name: "Slack",
      href: "https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg",
    },
    {
      name: "Github",
      href: "https://github.com/514-labs/moose",
    },
    {
      name: "X",
      href: "https://x.com/514hq",
    },
    {
      name: "LinkedIn",
      href: "https://www.linkedin.com/company/fiveonefour",
    },
  ],
};

const FooterContentContainer = () => {
  return (
    <div className="flex flex-row justify-evenly w-full items-start mb-4">
      <FooterNavItem
        item={{ name: "FiveOneFour", href: "https://fiveonefour.com" }}
        className="mb-0 w-full"
      >
        fiveonefour
      </FooterNavItem>
      {Object.entries(footerContent).map(([key, value]) => (
        <div className="flex flex-col w-full">
          <Text className="flex flex-row capitalize justify-start mb-0 self-start">
            {key}
          </Text>
          {value.map((item) => (
            <FooterNavItem
              item={item}
              className="my-0 py-0 text-muted-foreground"
              textClassName="text-muted-foreground mb-0"
            >
              {item.name}
            </FooterNavItem>
          ))}
        </div>
      ))}
    </div>
  );
};

export const FooterDisclaimerContainer = () => {
  const disclaimer = {
    rights: "2024 All rights reserved",
  };

  return (
    <div className="flex flex-row justify-between w-full items-center border-t">
      <Text className="grow">{disclaimer.rights}</Text>
      <ThemeToggle />
    </div>
  );
};

export const FooterContent = () => {
  return (
    <FullWidthContentContainer className="px-0 xl:px-6 flex flex-col gap-y-4">
      <FooterContentContainer />
      <FooterDisclaimerContainer />
    </FullWidthContentContainer>
  );
};

export default FooterSection;
