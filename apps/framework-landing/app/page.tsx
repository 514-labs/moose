import { cn } from "@/lib/utils";
import { ReactNode } from "react";
import { Button, buttonVariants } from "@/components/ui/button";
import { VariantProps } from "class-variance-authority";
import { EmailSection } from "@/app/sections/EmailSection";
import { LooseMooseSection } from "@/app/sections/home/LooseMooseSection";
import { FooterSection } from "@/app/sections/FooterSection";
import { HeroSection } from "./sections/home/HeroSection";
import { WhyMooseSection } from "./sections/home/WhyMooseSection";
import { WhatIsMooseSection } from "./sections/home/WhatIsMooseSection";
import { TemplatesSection } from "./sections/home/TemplatesSection";
import { FeaturesSection } from "./sections/home/FeaturesSection";
import { BuiltOnSection } from "./BuiltOnSection";
import { SecondaryCTASection } from "./sections/home/SecondaryCTASection";
import { GetMooseCTASection } from "./sections/home/GetMooseCTASection";
import { Text } from "@/components/typography/standard";

export const PlaceholderImage = ({ className }: { className?: string }) => {
  return <div className={cn("relative ", className)}> </div>;
};

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
        className
      )}
    >
      {children}
    </div>
  );
};

export interface CTAButtonProps extends VariantProps<typeof buttonVariants> {
  className?: string;
  children: ReactNode;
}

export const CTAButton = ({ className, children, variant }: CTAButtonProps) => {
  return (
    <Button
      size={"lg"}
      variant={variant}
      className="px-6 h-full font-normal border-primary"
    >
      <Text
        className={cn(
          variant === "outline" ? "text-primary" : "text-primary-foreground",
          className
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
