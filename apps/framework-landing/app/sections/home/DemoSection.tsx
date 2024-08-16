import getStarted from "/videos/cropped-demo.mp4";
import {
  Section,
  Grid,
  FullWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import dynamic from "next/dynamic";
const Video = dynamic(() => import("next-video"), { ssr: false });
import React from "react";

export const DemoSection = () => {
  return (
    <Section className="max-w-5xl mx-auto px-5">
      <Grid>
        <FullWidthContentContainer>
          <div className="p-[3px] rounded-2xl md:aspect-video bg-gradient-to-b from-[#5A5A5A] to-[#151515]">
            {/* <Image
          className="object-cover"
          src={"/images/demo/man_hero_upscale_forest.webp"}
          alt="lifestyleimg"
          fill
        ></Image> */}
            <div className="bg-background rounded-2xl">
              <Video
                src={getStarted}
                className="aspect-video overflow-hidden rounded-2xl"
                theme="microvideo"
              />
            </div>
          </div>
        </FullWidthContentContainer>
      </Grid>
    </Section>
  );
};
