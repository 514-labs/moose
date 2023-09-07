import { Metadata } from "next";
import { HeroTextMainComponent } from "./HeroTextMainComponent";
import { HeroTextSubComponent } from "./HeroTextSubComponent";
import { RightsComponent } from "./RightsComponent";
import { LogoComponent } from "./LogoComponent";
import { CTASection } from "./CTASection";

export const metadata: Metadata = {
  title: "Igloo | Data-intensive web apps",
  openGraph: {
    images: "/og_image_person_02_1x.webp"
  }
};


const Nav = () => {
  return (
    <div className="flex flex-row justify-between z-20 fixed text-typography-primary sm:p-8 sm: p-4">
      igloo
    </div>
  )
}

export default function Home() {
  return (
    <div className="h-full flex ">
      <div className="invisible sm:visible sm:w-full sm:h-full sm:fixed bg-[url('/bg_igloo_image_person_02_4x.webp')] bg-bottom bg-cover brightness-50 "/>
      <Nav />
      <div className="z-10 flex">
        <div className="flex flex-col xl:justify-center xl:content-center">
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
