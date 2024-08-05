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
    <Section className="xl:mx-auto xl:px-16 2xl:px-24 3xl:px-64 px-20">
      <Grid>
        <FullWidthContentContainer>
          <div className="mx-10 p-[3px] rounded-[30px] md:aspect-video bg-gradient-to-b from-[#5A5A5A] to-[#151515]">
            {/* <Image
          className="object-cover"
          src={"/images/demo/man_hero_upscale_forest.webp"}
          alt="lifestyleimg"
          fill
        ></Image> */}
            <div className="bg-background rounded-[27px] p-1">
              <Video
                src={getStarted}
                className="aspect-video overflow-hidden rounded-[26px]"
                theme="microvideo"
              />
            </div>
          </div>
        </FullWidthContentContainer>
      </Grid>
    </Section>
  );
};
