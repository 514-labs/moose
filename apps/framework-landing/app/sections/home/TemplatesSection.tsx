import Link from "next/link";
import { Fragment } from "react";

import { CTABar } from "../../page";
import {
  Section,
  FullWidthContentContainer,
  ThirdWidthContentContainer,
  Grid,
} from "design-system/components/containers";
import { Display, Heading, Text } from "design-system/typography";
import Image from "next/image";
import { TrackCtaButton } from "../../trackable-components";

const content = {
  title: "Use cases & templates",
  templates: [
    {
      title: "Product Analytics",
      imageSrc: "/images/templates/mjs-img-product-1.svg",
      description:
        "Capture user events and derive actionable insights with our ready-to-deploy, end-to-end product analytics solution, powered by MooseJS and NextJS.",
      cta: {
        subject: "cta-product-analytics-template-view",
        label: "Learn More",
        href: "/templates/product-analytics",
      },
    },
    {
      title: "LLM Application",
      imageSrc: "/images/templates/mjs-img-product-2.svg",
      description:
        "Enhance your operational workflows using custom AI agents, trained specifically to handle tasks based on your proprietary data and business context.",
      cta: {
        subject: "cta-product-analytics-template-view",
        label: "Learn More",
        href: "/templates/llm-application",
      },
    },
    {
      title: "Data Warehouse",
      imageSrc: "/images/templates/mjs-img-product-3.svg",
      description:
        "Unify data across your business domains, creating a platform optimized for analysis and data-driven strategy.",
      cta: {
        subject: "cta-product-analytics-template-view",
        label: "Learn More",
        href: "/templates/data-warehouse",
      },
    },
  ],
};

export const TemplateHeaderSection = () => {
  return (
    <Section>
      <Grid>
        <FullWidthContentContainer>
          <Display> {content.title} </Display>
        </FullWidthContentContainer>
      </Grid>
    </Section>
  );
};

export const TemplatesSection = () => {
  return (
    <Section>
      <Grid className="gap-y-5 justify-center">
        {content.templates.map((template, index) => {
          return (
            <Fragment key={index}>
              <ThirdWidthContentContainer className=" xl:m-0 bg-muted aspect-[4/3]  flex flex-col item-center justify-center xl:order-1">
                <div className="relative h-3/5">
                  <Image
                    priority
                    src={template.imageSrc}
                    fill
                    alt="man in jacket"
                    sizes=" (max-width: 768px) 150vw, 25vw"
                  />
                </div>
              </ThirdWidthContentContainer>
              <ThirdWidthContentContainer
                key={index}
                className="flex flex-col xl:justify-start xl:order-4"
              >
                <Heading>{template.title}</Heading>
                <Text className="xl:grow">{template.description}</Text>
                <CTABar>
                  <Link className="flex flex-col" href={template.cta.href}>
                    <TrackCtaButton
                      name="Learn More"
                      subject={template.cta.subject}
                      className="grow"
                      variant={"outline"}
                    >
                      {template.cta.label}
                    </TrackCtaButton>
                  </Link>
                </CTABar>
              </ThirdWidthContentContainer>
            </Fragment>
          );
        })}
      </Grid>
    </Section>
  );
};
