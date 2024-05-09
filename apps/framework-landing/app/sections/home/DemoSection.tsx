import getStarted from "/videos/Demo-video.mp4";
import { Section } from "@514labs/design-system/components/containers";
import dynamic from "next/dynamic";
const Video = dynamic(() => import("next-video"), { ssr: false });
import Image from "next/image";
import React from "react";

export const DemoSection = () => {
  return (
    <Section className="px-0 md:px-5">
      <div className="bg-muted p-5 md:p-24 md:aspect-video relative">
        <Image
          src={"/images/demo/img_man_hero.webp"}
          alt="lifestyleimg"
          objectFit="cover"
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
