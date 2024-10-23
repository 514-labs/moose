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
    <Section className="max-w-5xl mx-auto">
      <Grid>
        <FullWidthContentContainer>
          <Video
            src={getStarted}
            className="border-2 aspect-video overflow-hidden rounded-2xl mt-1"
            theme="microvideo"
          />
        </FullWidthContentContainer>
      </Grid>
    </Section>
  );
};
