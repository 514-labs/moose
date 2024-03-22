"use client";

import {
  FullWidthContentContainer,
  Section,
  Grid,
} from "@/components/containers/page-containers";
import { SuperDisplay } from "@/components/typography/standard";

export const EmailSection = () => {
  return (
    <Section className="text-center mb-0 lg:mb-0 2xl:mb-0 ">
      <Grid>
        <FullWidthContentContainer>
          <SuperDisplay> hello@moosejs.dev </SuperDisplay>
        </FullWidthContentContainer>
      </Grid>
    </Section>
  );
};
