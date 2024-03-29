import Link from "next/link";

import {
  FullWidthContentContainer,
  Grid,
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
      className={cn(
        "text-foreground flex flex-row justify-end lg:px-5",
        className
      )}
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
          <FooterNavItem item={item} key={item.name} className="md:p-5">
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
  };

  return (
    <>
      <Text className="my-1.5 grow">{disclaimer.rights}</Text>
      <Text className="my-1.5 mx-5">{disclaimer.by}</Text>
      <ThemeToggle />
    </>
  );
};

export const FooterContent = () => {
  return (
    <FullWidthContentContainer className="flex flex-row">
      <FooterDisclaimerContainer />
    </FullWidthContentContainer>
  );
};

export default FooterSection;
