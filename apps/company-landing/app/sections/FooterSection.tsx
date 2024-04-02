import Link from "next/link";

import {
  FullWidthContentContainer,
  Section,
} from "design-system/components/containers";

import { Text } from "design-system/typography";

import { cn } from "design-system/utils";
import { ThemeToggle } from "design-system/components";

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
    <Link
      href={item.href}
      className={cn("text-foreground flex flex-row justify-end ", className)}
    >
      <Text> {children} </Text>
    </Link>
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
    <div className="flex flex-col grow justify-center items-start sm:items-center md:flex-row md:justify-between lg:justify-end col-span-12 lg:col-span-6">
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
    by: "By the folks at fiveonefour",
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
