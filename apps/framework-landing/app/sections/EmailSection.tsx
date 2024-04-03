"use client";

import {
  FullWidthContentContainer,
  Grid,
  Section,
} from "design-system/components/containers";
import { SuperDisplay } from "design-system/typography";

export const EmailSection = () => {
  const content = {
    email: "hello@moosejs.dev",
  };

  return (
    <Section className="text-center">
      <Grid>
        <FullWidthContentContainer>
          <SuperDisplay> {content.email} </SuperDisplay>
        </FullWidthContentContainer>
      </Grid>
    </Section>
  );
};
