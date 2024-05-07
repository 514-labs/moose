import getStarted from "/videos/Demo-video.mp4";
import { Section } from "design-system/components/containers";
import dynamic from "next/dynamic";
const Video = dynamic(() => import("next-video"), { ssr: false });
import Image from "next/image";

export const DemoSection = () => {
  return (
    <Section>
      <div className="bg-muted py-36 px-40 aspect-video relative">
        <Image
          src={"/images/demo/img_man_hero.webp"}
          alt="lifestyleimg"
          fill
        ></Image>
        <Video
          src={getStarted}
          className="aspect-video rounded-sm overflow-hidden"
        />
      </div>
    </Section>
  );
};
