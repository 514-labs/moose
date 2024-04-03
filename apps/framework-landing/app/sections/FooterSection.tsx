import {
  FullWidthContentContainer,
  Section,
} from "design-system/components/containers";

import { Text } from "design-system/typography";

import { cn } from "design-system/utils";
import { ThemeToggle } from "design-system/components";
import { TrackLink } from "design-system/trackable-components";

export const FooterSection = () => {
  return (
    <Section>
      <FullWidthContentContainer className="">
        <FooterContent />
      </FullWidthContentContainer>
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
    by: "By the folks at fiveonefour",
    linkedin: {
      href: "https://www.linkedin.com/company/fiveonefour",
      name: "LinkedIn",
    },
    x: {
      href: "https://twitter.com/514hq",
      name: "X (prev. Twitter)",
    },
  };

  return (
    <>
      <Text className="grow">{disclaimer.rights}</Text>
      <Text>{disclaimer.by}</Text>
      <FooterNavItem item={disclaimer.linkedin}>
        {disclaimer.linkedin.name}
      </FooterNavItem>
      <FooterNavItem item={disclaimer.x}>{disclaimer.x.name}</FooterNavItem>
      <ThemeToggle />
    </>
  );
};

export const FooterContent = () => {
  return (
    <FullWidthContentContainer className="flex flex-col items-start md:flex-row md:items-center gap-x-5">
      <FooterDisclaimerContainer />
    </FullWidthContentContainer>
  );
};

export default FooterSection;
