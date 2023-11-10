import { Metadata } from "next";
import React from "react";
import { FooterSection } from "./sections/home/FooterSection";
import { CTASection } from "./sections/home/CTASection";
import { HowItWorksSection } from "./sections/home/HowItWorksSection";
import { HeroSection } from "./sections/home/HeroSection";
import { FeatureSection } from "./sections/home/FeatureSection";

export const metadata: Metadata = {
  title: "Igloo | Build Data-intensive apps with ease",
  openGraph: {
    images: "/og_igloo_4x.webp"
  }
};

export default function Home() {
  return (
    <div className="h-full">
      {}
      < HeroSection/>
      
      {}
      <FeatureSection />

      {}
      <HowItWorksSection />
      
      {}
      <CTASection />

      {}
      <FooterSection />
    </div>
  );
}
