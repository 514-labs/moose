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
    <Section className="md:my-0">
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
}: {
  item: { name: string; href: string };
  children: string;
  className?: string;
}) => {
  return (
    <TrackLink
      name="Footer Nav"
      subject={item.name}
      href={item.href}
      className={cn("text-foreground flex flex-row justify-end ", className)}
    >
      <Text> {children} </Text>
    </TrackLink>
  );
};

export const FooterDisclaimerContainer = () => {
  const disclaimer = {
    rights: "2024 All rights reserved",
    slack: {
      href: "https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg",
      name: "Slack",
    },
    github: {
      href: "https://github.com/514-labs/moose",
      name: "Github",
    },
  };

  return (
    <>
      <Text className="grow">{disclaimer.rights}</Text>

      <FooterNavItem item={disclaimer.slack}>
        {disclaimer.slack.name}
      </FooterNavItem>
      <FooterNavItem item={disclaimer.github}>
        {disclaimer.github.name}
      </FooterNavItem>
      <ThemeToggle />
    </>
  );
};

export const FooterContent = () => {
  return (
    <FullWidthContentContainer className="px-0 xl:px-6 flex flex-col items-center md:flex-row sm:items-center gap-x-5 mx-auto xl:max-w-screen-xl">
      <FooterDisclaimerContainer />
    </FullWidthContentContainer>
  );
};

export default FooterSection;
