import Link from "next/link";
import { Fragment, Suspense } from "react";

import { CTABar } from "../../page";
import {
  Section,
  HalfWidthContentContainer,
  ThirdWidthContentContainer,
  Grid,
} from "@514labs/design-system-components/components/containers";
import {
  Heading,
  HeadingLevel,
  Text,
} from "@514labs/design-system-components/typography";
import { TrackCtaButton } from "../../trackable-components";
import { TemplateImg } from "./TemplateImg";
import React from "react";

const content = {
  title: "Templates",
  description:
    "Full-stack data & analytics application templates to get you started quickly",
  templates: [
    {
      title: "Product Analytics",
      imageSrcLight: "/images/diagrams/img-diagram-PA-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-PA-dark.svg",
      description:
        "Capture user journeys and derive actionable insights to optimize your product development",
      cta: {
        subject: "cta-product-analytics-template-view",
        label: "Learn More",
        href: "/templates/product-analytics",
      },
    },
    {
      title: "LLM Application",
      imageSrcLight: "/images/diagrams/img-diagram-LLM-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-LLM-dark.svg",
      description:
        "Optimize AI automations powered by RAG on your own data to create innovative end user experiences",
      cta: {
        subject: "cta-product-analytics-template-view",
        label: "Learn More",
        href: "/templates/llm-application",
      },
    },
    {
      title: "Data Warehouse",
      imageSrcLight: "/images/diagrams/img-diagram-DW-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-DW-dark.svg",
      description:
        "Integrate data across business domains into a data warehouse with discoverable, consumable data products",
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
    <Section className="w-full relative mx-auto xl:max-w-screen-xl">
      <Grid>
        <HalfWidthContentContainer>
          <Heading> {content.title} </Heading>
          <Heading className="text-muted-foreground" level={HeadingLevel.l2}>
            {" "}
            {content.description}{" "}
          </Heading>
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};

export const TemplatesSection = () => {
  return (
    <Section className="w-full relative mx-auto xl:my-10 xl:max-w-screen-xl 2xl:my-0">
      <Grid className="gap-y-5 justify-center">
        {content.templates.map((template, index) => {
          return (
            <Fragment key={index}>
              <ThirdWidthContentContainer
                key={index}
                className="flex flex-col xl:justify-start xl:order-4 border border-muted-foreground rounded-3xl p-5"
              >
                <div className="relative aspect-square my-0">
                  <Suspense fallback={<div>Loading...</div>}>
                    <TemplateImg
                      srcDark={template.imageSrcDark}
                      srcLight={template.imageSrcLight}
                      alt={template.title}
                    />
                  </Suspense>
                </div>
                <div className="my-5">
                  <Text className="my-0">{template.title}</Text>
                  <Text className="my-0 text-muted-foreground xl:grow">
                    {template.description}
                  </Text>
                </div>
                <CTABar className="my-5">
                  <Link className="flex flex-col" href={template.cta.href}>
                    <TrackCtaButton
                      name="Learn More"
                      subject={template.cta.subject}
                      className="grow"
                      variant={"outline"}
                      targetUrl={template.cta.href}
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
