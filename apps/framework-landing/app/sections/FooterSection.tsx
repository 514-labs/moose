import {
  FullWidthContentContainer,
  Section,
} from "@/components/containers/page-containers";

import Link from "next/link";
import { cn } from "@/lib/utils";
import { SmallText } from "@/components/typography/standard";

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
    <Link href={item.href} className={cn("text-foreground", className)}>
      <SmallText> {children} </SmallText>
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
    <div className="flex flex-col sm:flex-row sm:justify-between lg:justify-normal">
      {navigation.map((item) => {
        return (
          <FooterNavItem item={item} key={item.name} className="p-5">
            {item.name}
          </FooterNavItem>
        );
      })}
    </div>
  );
};

export const FooterNavContainer = () => {
  return <FooterNav />;
};

export const FooterDisclaimerContainer = () => {
  return (
    <div className="flex flex-row grow min-h-16">
      <div className="bg-primary aspect-square h-full min-h-16"></div>
      <div className="flex flex-col justify-center px-5 no-wrap">
        <SmallText className="my-0">2024 All rights rserved</SmallText>
        <SmallText className="my-0">
          Moose.js is by the folks at fiveonefour
        </SmallText>
      </div>
    </div>
  );
};

export const FooterContent = () => {
  return (
    <FullWidthContentContainer className="flex flex-col lg:flex-row">
      <FooterDisclaimerContainer />
      <FooterNavContainer />
    </FullWidthContentContainer>
  );
};
