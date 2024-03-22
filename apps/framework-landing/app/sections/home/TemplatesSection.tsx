import {
  ThirdWidthContentContainer,
  FullWidthContentContainer,
  Section,
  Grid,
} from "@/components/containers/page-containers";
import Link from "next/link";
import { Display, Heading, Text } from "@/components/typography/standard";
import { PlaceholderImage, CTABar, CTAButton } from "../../page";

export const TemplatesSection = () => {
  const content = {
    title: "Use cases & templates",
    templates: [
      {
        title: "Product Analytics",
        description:
          "Capture user events and derive actionable insights with our ready-to-deploy, end-to-end product analytics solution, powered by MooseJS and NextJS.",
        cta: {
          action: "cta-product-analytics-template-view",
          label: "Learn More",
          href: "/templates/product-analytics",
        },
      },
      {
        title: "LLM Application",
        description:
          "Enhance your operational workflows using custom AI agents, trained specifically to handle tasks based on your proprietary data and business context.",
        cta: {
          action: "cta-product-analytics-template-view",
          label: "Learn More",
          href: "/templates/llm-application",
        },
      },
      {
        title: "Data Warehouse",
        description:
          "Unify data across your business domains, creating a platform optimized for analysis and data-driven strategy.",
        cta: {
          action: "cta-product-analytics-template-view",
          label: "Learn More",
          href: "/templates/data-warehouse",
        },
      },
    ],
  };

  return (
    <>
      <Section>
        <Grid>
          <FullWidthContentContainer>
            <Display> {content.title} </Display>
          </FullWidthContentContainer>
        </Grid>
      </Section>
      <Section>
        <Grid className="gap-y-5">
          <ThirdWidthContentContainer className="xl:order-1">
            <PlaceholderImage className=" bg-muted aspect-[4/3]" />
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="flex flex-col justify-center xl:order-4">
            <Heading> {content.templates[0].title} </Heading>
            <Text> {content.templates[0].description} </Text>
            <CTABar>
              <Link
                className="flex flex-col"
                href={content.templates[0].cta.href}
              >
                <CTAButton className="grow" variant={"outline"}>
                  {content.templates[0].cta.label}
                </CTAButton>
              </Link>
            </CTABar>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className=" xl:m-0 xl:order-1">
            <PlaceholderImage className=" bg-muted aspect-[4/3]" />
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="flex flex-col justify-center xl:m-0  xl:order-4">
            <Heading> {content.templates[1].title} </Heading>
            <Text> {content.templates[1].description} </Text>
            <CTABar>
              <Link
                className="flex flex-col"
                href={content.templates[1].cta.href}
              >
                <CTAButton variant={"outline"}>
                  {content.templates[1].cta.label}
                </CTAButton>
              </Link>
            </CTABar>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="xl:order-1">
            <PlaceholderImage className=" bg-muted aspect-[4/3]" />
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="flex flex-col justify-center xl:order-4">
            <Heading> {content.templates[2].title} </Heading>
            <Text> {content.templates[2].description} </Text>
            <CTABar>
              <Link
                className="flex flex-col"
                href={content.templates[2].cta.href}
              >
                <CTAButton variant={"outline"}>
                  {content.templates[2].cta.label}
                </CTAButton>
              </Link>
            </CTABar>
          </ThirdWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
