// Sadly, this layout page is required in all our post directories

import {
  FullWidthContentContainer,
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";

export default function MdxLayout({
  children,
  meta,
  breadcrumbs,
}: {
  children: React.ReactNode;
  meta: React.ReactNode;
  breadcrumbs: React.ReactNode;
}) {
  // Create any shared layout or styles here
  return (
    <Section>
      <Grid>
        <FullWidthContentContainer>{breadcrumbs}</FullWidthContentContainer>
        <HalfWidthContentContainer>
          <div className="sticky top-36">{meta}</div>
        </HalfWidthContentContainer>
        <HalfWidthContentContainer>{children}</HalfWidthContentContainer>
      </Grid>
    </Section>
  );
}
