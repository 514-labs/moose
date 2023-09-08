import { Metadata } from "next";
import { HeroTextMainComponent } from "./HeroTextMainComponent";
import { HeroTextSubComponent } from "./HeroTextSubComponent";
import { CTASection } from "./CTASection";
import React from "react";
import { BackgroundImage } from "./BackgroundImage";
import { Nav } from "./Nav";

export const metadata: Metadata = {
  title: "Igloo | Build Data-intensive apps with ease",
  openGraph: {
    images: "/og_igloo_image_person_012_2x.webp"
  }
};

export default function Home() {
  return (
    <div className="h-full w-screen flex grow-1">
      <BackgroundImage />
      <Nav />
      <div className="z-10 flex grow">
        <div className="flex flex-col xl:justify-center xl:content-center grow">
          <div className="py-20 flex grow md:flex-nowrap flex-wrap flex-col text-typography-primary content-center justify-center relative">
            <div className="flex md:flex-nowrap flex-wrap flex-row text-typography-primary content-center justify-center z-10 sm:p-8 p-4 shrink-0 min-h-[50%]">
              <HeroTextMainComponent />
              <HeroTextSubComponent />
            </div>
            <div className="visibile absolute h-full w-full sm:invisible bg-[url('/bg_igloo_image_person_02_4x.webp')] bg-bottom bg-cover brightness-50 "/>
          </div>
          <div>
            < CTASection />
          </div>  d
      </div>
    </div>
  </div>
  );
}
