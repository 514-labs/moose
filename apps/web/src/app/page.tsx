import { Metadata } from "next";
import { HeroTextMainComponent } from "./HeroTextMainComponent";
import { HeroTextSubComponent } from "./HeroTextSubComponent";
import { CTASection } from "./CTASection";
import React from "react";
import { BackgroundImage } from "./BackgroundImage";
import { Nav } from "./Nav";

export const metadata: Metadata = {
  title: "Igloo | Data-intensive web apps",
  openGraph: {
    images: "/og_image_person_02_1x.webp"
  }
};

export default function Home() {
  return (
    <div className="h-full w-screen flex grow-1">
      <BackgroundImage />
      <Nav />
      <div className="z-10 flex grow">
        <div className="flex flex-col xl:justify-center xl:content-center grow">
          <div className="flex grow md:flex-nowrap flex-wrap flex-col text-typography-primary content-center justify-center relative">
            <div className="flex md:flex-nowrap flex-wrap flex-row text-typography-primary content-center justify-center z-10 sm:p-8 p-4">
              <HeroTextMainComponent />
              <HeroTextSubComponent />
            </div>
            <div className="visibile absolute h-full w-full sm:invisible bg-[url('/bg_igloo_image_person_02_4x.webp')] bg-bottom bg-cover brightness-50 "/>
          </div>
          <div>
            < CTASection />
          </div>  
      </div>
    </div>
  </div>
  );
}
