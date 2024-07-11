"use client";

import { horizontalLoop } from "../../../lib/animation-helpers";
import { Section } from "@514labs/design-system-components/components/containers";
import { BannerDisplay } from "@514labs/design-system-components/typography";
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
    <Section gutterless className="hidden w-full md:my-0 md:block">
      <BannerDisplay className="flex flex-row w-full inset-x-0 bottom-0">
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
