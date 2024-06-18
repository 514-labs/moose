"use client";

import {
  FullWidthContentContainer,
  Grid,
  Section,
} from "@514labs/design-system-components/components/containers";
import { Display } from "@514labs/design-system-components/typography";

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
