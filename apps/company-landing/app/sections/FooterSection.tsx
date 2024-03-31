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
      className={cn(
        "text-foreground flex flex-row justify-end lg:px-5",
        className,
      )}
    >
      <Text> {children} </Text>
    </Link>
  );
};

export const FooterDisclaimerContainer = () => {
  const disclaimer = {
    rights: "2024 All rights reserved",
    by: "By the folks at fiveonefour",
  };

  return (
    <>
      <Text className="my-1.5 2xl:my-0 grow">{disclaimer.rights}</Text>
      <Text className="my-1.5 2xl:my-0 mx-5">{disclaimer.by}</Text>
      <ThemeToggle />
    </>
  );
};

export const FooterContent = () => {
  return (
    <FullWidthContentContainer className="flex flex-row ">
      <FooterDisclaimerContainer />
    </FullWidthContentContainer>
  );
};

export default FooterSection;
