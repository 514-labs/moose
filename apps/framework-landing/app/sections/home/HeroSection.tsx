import { CTABar } from "../../page";
import {
  Section,
  Grid,
} from "@514labs/design-system/components/containers";
import { Heading, HeadingLevel} from "@514labs/design-system/typography";
import Image from "next/image";
import {
  TrackCtaButton,
  TrackableCodeSnippet,
} from "../../trackable-components";

export const HeroSection = () => {
  const content = {
    tagLine: "Your own scalable data products in minutes",
    description:
      "An open source developer framework for your data & analytics stack",
    ctas: [
      {
        href: "https://docs.moosejs.com/getting-started/new-project",
        action: "cta-early-access",
        label: "Get Started",
        variant: "default",
      },
      {
        href: "https://docs.moosejs.com/",
        action: "cta-early-access",
        label: "View Docs ",
        variant: "outline",
      },
    ],
  };

  return (
    <>
      <Section className="mt-12 lg:mt-12 2xl:mt-24 px-5">
        <Grid>
          <FullWidthContentContainer className="">
            <div>
              <Heading> {content.tagLine} </Heading>
              <Heading
                level={HeadingLevel.l2}
                className="text-muted-foreground"
              >
                {" "}
                {content.description}{" "}
              </Heading>
            </div>
            <CTABar className="mb-5">
              {content.ctas.map((cta, index) => (
                <Link key={index} href={cta.href}>
                  <TrackCtaButton
                    name={cta.label}
                    subject={cta.label}
                    variant={cta.variant as "default" | "outline"}
                  >
                    {cta.label}
                  </TrackCtaButton>
                </Link>
              ))}
            </CTABar>
          </FullWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
