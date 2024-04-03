import { ReactNode } from "react";

import { VariantProps } from "class-variance-authority";
import { EmailSection } from "./sections/EmailSection";
import { LooseMooseSection } from "./sections/home/LooseMooseSection";
import { FooterSection } from "./sections/FooterSection";
import { HeroSection } from "./sections/home/HeroSection";
import { WhyMooseSection } from "./sections/home/WhyMooseSection";
import { WhatIsMooseSection } from "./sections/home/WhatIsMooseSection";
import {
  TemplateHeaderSection,
  TemplatesSection,
} from "./sections/home/TemplatesSection";
import { FeaturesSection } from "./sections/home/FeaturesSection";
import { BuiltOnSection } from "./sections/home/BuiltOnSection";
import { SecondaryCTASection } from "./sections/home/SecondaryCTASection";
import { GetMooseCTASection } from "./sections/home/GetMooseCTASection";
import { cn } from "design-system/utils";

import { Button, buttonVariants } from "design-system/components";

import { Text } from "design-system/typography";

export const CTAText = ({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <div
      className={cn(
        "text-center md:text-start text-primary text-4xl bg-muted rounded-md py-5 px-10 text-nowrap",
        className,
      )}
    >
      {children}
    </div>
  );
};

export interface CTAButtonProps extends VariantProps<typeof buttonVariants> {
  className?: string;
  children: ReactNode;
  onClick?: () => void;
}

export const CTAButton = ({
  className,
  children,
  variant,
  onClick,
}: CTAButtonProps) => {
  return (
    <Button
      size={"lg"}
      variant={variant}
      className=" h-full font-normal border-primary"
      onClick={onClick}
    >
      <Text
        className={cn(
          variant === "outline" ? "text-primary" : "text-primary-foreground",
          className,
        )}
      >
        {children}
      </Text>
    </Button>
  );
};

export const CTABar = ({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <div className={cn("flex flex-col md:flex-row gap-5", className)}>
      {children}
    </div>
  );
};

export default function Home() {
  return (
    <main className="min-h-screen">
      <HeroSection />
      <WhyMooseSection />
      <WhatIsMooseSection />
      <FeaturesSection />
      <TemplateHeaderSection />
      <TemplatesSection />
      <BuiltOnSection />
      <LooseMooseSection />
      <SecondaryCTASection />
      <GetMooseCTASection />
      <FooterSection />
      <EmailSection />
    </main>
  );
}
