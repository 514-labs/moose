import { Metadata } from "next";
import { HeroTextMainComponent } from "./HeroTextMainComponent";
import { HeroTextSubComponent } from "./HeroTextSubComponent";
import { RightsComponent } from "./RightsComponent";
import { LogoComponent } from "./LogoComponent";
import { CTASection } from "./CTASection";

export const metadata: Metadata = {
  title: "Igloo | Data-intensive web apps",
};

export default function Home() {
  return (
    <div className="h-full flex ">
      <div className="w-full h-full fixed bg-[url('/cover_bg_woman_2_1x.webp')] bg-bottom bg-cover brightness-50 "/>
      <div className="z-10 flex">
        <div className="flex sm:p-8 p-4 flex-col xl:justify-center xl:content-center">
        <div className="flex grow md:flex-nowrap flex-wrap flex-col text-typography-primary content-center justify-center">
          <div className="flex md:flex-nowrap flex-wrap flex-row text-typography-primary content-center justify-center z-10">
            <HeroTextMainComponent />
            <HeroTextSubComponent />
          </div>
        </div>
        <div>
            < CTASection />
        </div>
      </div>
    </div>
  </div>
  );
}
