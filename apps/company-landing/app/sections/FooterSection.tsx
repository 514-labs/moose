import {
  FullWidthContentContainer,
  Section,
} from "@514labs/design-system/components/containers";

import { Text } from "@514labs/design-system/typography";

import { cn } from "@514labs/design-system/utils";
import { ThemeToggle } from "@514labs/design-system/components";
import { TrackLink } from "@514labs/design-system/trackable-components";

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

export const FooterNav = () => {
  const navigation = [
    { name: "docs", href: "https://docs.moosejs.dev" },
    { name: "templates", href: "/templates" },
    { name: "blog", href: "https://blog.fiveonefour.com/" },
    { name: "github", href: "https://github.com/514-labs/moose" },
    { name: "community", href: "/community" },
  ];

  return (
    <div className="flex flex-col grow justify-center items-start sm:items-left md:flex-row md:justify-between lg:justify-end col-span-12 lg:col-span-6">
      {navigation.map((item) => {
        return (
          <FooterNavItem item={item} key={item.name}>
            {item.name}
          </FooterNavItem>
        );
      })}
      <ThemeToggle />
    </div>
  );
};

export const FooterNavContainer = () => {
  return <FooterNav />;
};

export const FooterDisclaimerContainer = () => {
  const disclaimer = {
    rights: "2024 All rights reserved",
    linkedin: {
      href: "https://www.linkedin.com/company/fiveonefour/",
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
    <FullWidthContentContainer className="flex flex-col items-center md:flex-row sm:items-center gap-x-5">
      <FooterDisclaimerContainer />
    </FullWidthContentContainer>
  );
};

export default FooterSection;
