import Link from "next/link";
import { Fragment, Suspense } from "react";

import { CTABar } from "../../page";
import {
  Section,
  FullWidthContentContainer,
  ThirdWidthContentContainer,
  Grid,
} from "@514labs/design-system-components/components/containers";
import {
  Heading,
  HeadingLevel,
  Text,
} from "@514labs/design-system-components/typography";
import { TrackButton } from "@514labs/design-system-components/trackable-components";
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
        "Capture clickstream events and analyze user journeys in real-time",
      cta: {
        subject: "cta-product-analytics-template-view",
        label: "Learn More",
        href: "/templates/product-analytics",
      },
    },
    {
      title: "Observability Logging",
      imageSrcLight: "/images/diagrams/img-diagram-DW-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-DW-dark.svg",
      description:
        "Collect, parse, organize, and search OpenTelemetry logs from your apps",
      cta: {
        subject: "cta-product-analytics-template-view",
        label: "Learn More",
        href: "/templates/observability-logging",
      },
    },
    {
      title: "Realtime Leaderboards",
      imageSrcLight: "/images/diagrams/img-diagram-DW-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-DW-dark.svg",
      description:
        "Add dynamic leaderboards to your user facing apps, powered by live data",
      cta: {
        subject: "cta-product-analytics-template-view",
        label: "Learn More",
        href: "/templates/realtime-leaderboard",
      },
    },
  ],
};

export const TemplateHeaderSection = () => {
  return (
    <Section className="w-full relative mx-auto xl:max-w-screen-xl">
      <Grid>
        <FullWidthContentContainer>
          <Heading> {content.title} </Heading>
          <Heading className="text-muted-foreground" level={HeadingLevel.l3}>
            {" "}
            {content.description}{" "}
          </Heading>
        </FullWidthContentContainer>
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
                className="flex flex-col xl:justify-start xl:order-4 p-5 border rounded-3xl"
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
                    <TrackButton
                      name="Learn More"
                      subject={template.cta.subject}
                      className="grow"
                      targetUrl={template.cta.href}
                    >
                      {template.cta.label}
                    </TrackButton>
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
