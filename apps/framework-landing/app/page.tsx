import { FooterSection } from "./sections/FooterSection";
import { HeroSection } from "./sections/home/HeroSection";
import { PrimitivesCode } from "./sections/home/PrimitivesCode";
import { FeaturesSection } from "./sections/home/FeaturesSection";
import { DemoSection } from "./sections/home/DemoSection";
import { SecondaryCTASection } from "./sections/home/HostWithBorealSection";
import { cn } from "@514labs/design-system-components/utils";
import { WhatIsMooseFor } from "./sections/home/WhatIsMooseForV2";
import MooseValuePropSection from "./sections/home/MooseValuePropSection";

export const CTABar = ({
  className,
  children,
}: {
  className?: string;
  children: React.ReactNode;
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
      <HeroSection className="max-w-5xl mx-auto flex flex-col items-center px-5 my-16 sm:my-32 py-10 pb-20" />
      <DemoSection />
      <MooseValuePropSection />
      <PrimitivesCode />
      <FeaturesSection />
      <WhatIsMooseFor />
      <SecondaryCTASection />
      <FooterSection />
    </main>
  );
}
