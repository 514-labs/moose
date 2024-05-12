"use client";

import {
  FullWidthContentContainer,
  Grid,
  Section,
} from "@514labs/design-system/components/containers";
import { Display } from "@514labs/design-system/typography";

export const EmailSection = () => {
  const content = {
    email: "hello@moosejs.dev",
  };

  return (
    <Section className="text-center">
      <Grid>
        <FullWidthContentContainer>
          <Display> {content.email} </Display>
        </FullWidthContentContainer>
      </Grid>
    </Section>
  );
};
