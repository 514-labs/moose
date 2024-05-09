import Link from "next/link";
import { Fragment, Suspense } from "react";

import { CTABar } from "../../page";
import {
  Section,
  FullWidthContentContainer,
  ThirdWidthContentContainer,
  Grid,
} from "@514labs/design-system/components/containers";
import { Display, Heading, Text } from "@514labs/design-system/typography";
import { TrackCtaButton } from "../../trackable-components";
import { TemplateImg } from "./TemplateImg";

const content = {
  title: "Use cases & templates",
  templates: [
    {
      title: "Product Analytics",
      imageSrcLight: "/images/templates/img-product-1-light.svg",
      imageSrcDark: "/images/templates/img-product-1-dark.svg",
      description:
        "Capture user journeys and derive actionable insights to optimize your product development.",
      cta: {
        subject: "cta-product-analytics-template-view",
        label: "Learn More",
        href: "/templates/product-analytics",
      },
    },
    {
      title: "LLM Application",
      imageSrcLight: "/images/templates/img-product-2-light.svg",
      imageSrcDark: "/images/templates/img-product-2-dark.svg",
      description:
        "Optimize AI automations powered by RAG on your own data to create innovative end user experiences.",
      cta: {
        subject: "cta-product-analytics-template-view",
        label: "Learn More",
        href: "/templates/llm-application",
      },
    },
    {
      title: "Data Warehouse",
      imageSrcLight: "/images/templates/img-product-3-light.svg",
      imageSrcDark: "/images/templates/img-product-3-dark.svg",
      description:
        "Integrate data across business domains into a data warehouse with discoverable, consumable data products.",
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
                  <Suspense fallback={<div>Loading...</div>}>
                    <TemplateImg
                      srcDark={template.imageSrcDark}
                      srcLight={template.imageSrcLight}
                      alt={template.title}
                    />
                  </Suspense>
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
