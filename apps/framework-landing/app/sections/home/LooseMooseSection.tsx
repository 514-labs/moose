"use client";

import { horizontalLoop } from "../../../lib/animation-helpers";
import { Section } from "design-system/components/containers";
import { BannerDisplay } from "design-system/typography";
import { useRef } from "react";
import { useLayoutEffect } from "react";

export const LooseMooseSection = () => {
  const bannerItem1 = useRef(null);
  const bannerItem2 = useRef(null);

  const content = {
    banner: "Let the moose loose",
  };

  useLayoutEffect(() => {
    horizontalLoop([bannerItem1.current, bannerItem2.current], {
      repeat: -1,
      paused: false,
      speed: 3,
    });
  });

  return (
    <Section gutterless className="hidden md:block">
      <BannerDisplay className="flex flex-row">
        <span className="px-20 will-change-transform" ref={bannerItem1}>
          {content.banner}
        </span>
        <span className="px-20 will-change-transform" ref={bannerItem2}>
          {content.banner}
        </span>
      </BannerDisplay>
    </Section>
  );
};
