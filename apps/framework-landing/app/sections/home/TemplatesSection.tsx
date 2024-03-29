import Link from "next/link";

import { PlaceholderImage, CTABar, CTAButton } from "../../page";
import {
  Section,
  FullWidthContentContainer,
  ThirdWidthContentContainer,
  Grid,
} from "design-system/components/containers";
import { Display, Heading, Text } from "design-system/typography";
import Image from "next/image";

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
        <Grid className="gap-y-5 justify-center">
          <ThirdWidthContentContainer className="xl:order-1 bg-muted aspect-[4/3] flex flex-col item-center justify-center">
            <div className="relative h-3/5">
              <Image
                priority
                src="/images/templates/mjs-img-product-1.svg"
                fill
                alt="man in jacket"
                sizes=" (max-width: 768px) 150vw, 25vw"
              />
            </div>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="flex flex-col xl:justify-start xl:order-4">
            <Heading>
              {" "}
              {content.templates[0] && content.templates[0].title}{" "}
            </Heading>
            <Text className="xl:grow">
              {" "}
              {content.templates[0] && content.templates[0].description}{" "}
            </Text>
            <CTABar>
              <Link
                className="flex flex-col"
                href={content.templates[0] ? content.templates[0].cta.href : ""}
              >
                <CTAButton className="grow" variant={"outline"}>
                  {content.templates[0] && content.templates[0].cta.label}
                </CTAButton>
              </Link>
            </CTABar>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className=" xl:m-0 bg-muted aspect-[4/3]  flex flex-col item-center justify-center xl:order-1">
            <div className="relative h-3/5">
              <Image
                priority
                src="/images/templates/mjs-img-product-2.svg"
                fill
                alt="man in jacket"
                sizes=" (max-width: 768px) 150vw, 25vw"
              />
            </div>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="flex flex-col xl:justify-start xl:m-0  xl:order-4">
            <Heading>
              {" "}
              {content.templates[1] && content.templates[1].title}{" "}
            </Heading>
            <Text className="xl:grow">
              {" "}
              {content.templates[1] && content.templates[1].description}{" "}
            </Text>
            <CTABar>
              <Link
                className="flex flex-col"
                href={content.templates[1] ? content.templates[1].cta.href : ""}
              >
                <CTAButton variant={"outline"}>
                  {content.templates[1] && content.templates[1].cta.label}
                </CTAButton>
              </Link>
            </CTABar>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="xl:order-1 bg-muted aspect-[4/3]  flex flex-col item-center justify-center">
            <div className="relative h-3/5">
              <Image
                priority
                src="/images/templates/mjs-img-product-3.svg"
                fill
                alt="man in jacket"
                sizes=" (max-width: 768px) 150vw, 25vw"
              />
            </div>
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="flex flex-col  xl:justify-start xl:order-4">
            <Heading>
              {" "}
              {content.templates[2] && content.templates[2].title}{" "}
            </Heading>
            <Text className="xl:grow">
              {" "}
              {content.templates[2] && content.templates[2].description}{" "}
            </Text>
            <CTABar>
              <Link
                className="flex flex-col"
                href={content.templates[2] ? content.templates[2].cta.href : ""}
              >
                <CTAButton variant={"outline"}>
                  {content.templates[2] && content.templates[2].cta.label}
                </CTAButton>
              </Link>
            </CTABar>
          </ThirdWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
