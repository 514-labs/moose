// Sadly, this layout page is required in all our post directories

import {
  FullWidthContentContainer,
  Grid,
  HalfWidthContentContainer,
  Section,
} from "@514labs/design-system-components/components/containers";
import FooterSection from "../../../sections/FooterSection";
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
    <Section className="px-6 w-full relative mx-auto xl:max-w-screen-xl">
      <Grid>
        <FullWidthContentContainer>{breadcrumbs}</FullWidthContentContainer>
        <HalfWidthContentContainer>
          <div className="sticky top-36">{meta}</div>
        </HalfWidthContentContainer>
        <HalfWidthContentContainer>{children}</HalfWidthContentContainer>
      </Grid>
      <FooterSection />
    </Section>
  );
}
