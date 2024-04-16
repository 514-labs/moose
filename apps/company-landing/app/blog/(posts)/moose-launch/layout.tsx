// Sadly, this layout page is required in all our post directories

import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";

export default function MdxLayout({
  children,
  meta,
}: {
  children: React.ReactNode;
  meta: React.ReactNode;
}) {
  // Create any shared layout or styles here
  return (
    <Section>
      <Grid>
        <HalfWidthContentContainer>
          <div className="sticky top-36">{meta}</div>
        </HalfWidthContentContainer>
        <HalfWidthContentContainer>{children}</HalfWidthContentContainer>
      </Grid>
    </Section>
  );
}
