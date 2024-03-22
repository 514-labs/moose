"use client";

import {
  FullWidthContentContainer,
  Section,
  Grid,
} from "@/components/containers/page-containers";
import { SuperDisplay } from "@/components/typography/standard";

export const EmailSection = () => {
  const content = {
    email: "hello@moosejs.dev",
  };

  return (
    <Section className="text-center mb-0 lg:mb-0 2xl:mb-0 ">
      <Grid>
        <FullWidthContentContainer>
          <SuperDisplay> {content.email} </SuperDisplay>
        </FullWidthContentContainer>
      </Grid>
    </Section>
  );
};
