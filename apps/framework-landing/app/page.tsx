import { ReactNode } from "react";

import { VariantProps } from "class-variance-authority";
import { DemoSection } from "./sections/home/DemoSection";
import { FooterSection } from "./sections/FooterSection";
import { HeroSection } from "./sections/home/HeroSection";
import { WhyMooseSection } from "./sections/home/WhyMooseSection";
import { MooseStackSection } from "./sections/home/MooseStackSection";
import {
  TemplateHeaderSection,
  TemplatesSection,
} from "./sections/home/TemplatesSection";
import { FeaturesSection } from "./sections/home/FeaturesSection";

import { SecondaryCTASection } from "./sections/home/SecondaryCTASection";
import { cn } from "@514labs/design-system-components/utils";

import {
  Button,
  buttonVariants,
} from "@514labs/design-system-components/components";

import { Text } from "@514labs/design-system-components/typography";
import React from "react";
import { UsecasesSection } from "./sections/home/UsecasesSection";
import { LooseMooseSection } from "./sections/home/LooseMooseSection";
import Diagram from "./spline";
import DiagramSection from "./sections/home/DiagramSection";

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
      className={cn(
        "h-full font-normal border-primary w-full sm:w-auto px-10 py-0 rounded-xl",
        className,
      )}
      onClick={onClick}
    >
      <Text
        className={cn(
          variant === "outline" ? "text-primary" : "text-primary-foreground",
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
    <main>
      <HeroSection />
      <DemoSection />
      <FeaturesSection />
      <UsecasesSection />
      <MooseStackSection />
      <WhyMooseSection />

      {/* <HowItWorksSection /> */}
      <DiagramSection />

      <TemplateHeaderSection />
      <TemplatesSection />
      <SecondaryCTASection />
      <FooterSection />
      <LooseMooseSection />
    </main>
  );
}
