import getStarted from "/videos/Demo-video.mp4";
import { Section } from "@514labs/design-system/components/containers";
import dynamic from "next/dynamic";
const Video = dynamic(() => import("next-video"), { ssr: false });
import Image from "next/image";
import React from "react";

export const DemoSection = () => {
  return (
    <Section className="px-0 md:px-0 xl:mx-auto xl:px-16 2xl:px-24 3xl:px-64">
      <div className="bg-muted px-12 py-24 md:px-36 lg:py-36 xl:py-36 md:aspect-video relative 2xl:px-56 4xl:px-64">
        <Image
          className="object-cover"
          src={"/images/demo/man_hero_upscale_forest.webp"}
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
