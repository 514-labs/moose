import {
  FullWidthContentContainer,
  Grid,
  Section,
  HalfWidthContentContainer,
} from "@514labs/design-system-components/components/containers";

import {
  Heading,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import React from "react";
import { Fragment, Suspense } from "react";

import { TemplateImg } from "../../sections/home/TemplateImg";
import { Badge } from "@514labs/design-system-components/components";

const mooseContent = {
  title: "When to use Moose",
  usecases: [
    {
      title: "Data-intensive apps",
      description:
        "Build an OLAP-centric backend to serve real-time leaderboards, charts, and metrics in your apps",
      badge: "Moose + OLTP",
      imageSrcLight: "/images/diagrams/img-diagram-data-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-data-dark.svg",
    },
    {
      title: "Enterprise data products",
      description:
        "Leverage Moose to build data services powering BI, AI/ML pipelines, and notebooks",
      badge: "Moose",
      imageSrcLight: "/images/diagrams/img-diagram-ent-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-ent-dark.svg",
    },
  ],
};

const notMooseContent = {
  title: "When to not use Moose",
  subtitle: "Transactional Apps",
  description:
    "If you’re building apps with high transactional workloads and CRUD operations, you’ll be better served by OLTP focused frameworks",
  badge: "OLTP",
  imageSrcLight: "/images/diagrams/img-diagram-standard-light.svg",
  imageSrcDark: "/images/diagrams/img-diagram-standard-dark.svg",
};

export const WhatIsntMooseFor = () => {
  return (
    <>
      <Section className="mx-auto xl:max-w-screen-xl">
        <FullWidthContentContainer>
          <Heading>{notMooseContent.title}</Heading>
        </FullWidthContentContainer>
        <Grid className="items-center">
          <HalfWidthContentContainer className="p-10">
            <div className="relative aspect-square my-0">
              <Suspense fallback={<div>Loading...</div>}>
                <TemplateImg
                  srcDark={notMooseContent.imageSrcDark}
                  srcLight={notMooseContent.imageSrcLight}
                  alt={notMooseContent.subtitle}
                />
              </Suspense>
            </div>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer className="flex flex-col gap-5 align-bottom p-10">
            <Badge className="w-fit p-2" variant={"outline"}>
              {notMooseContent.badge}
            </Badge>
            <Heading level={HeadingLevel.l2} className="my-0">
              {notMooseContent.subtitle}
            </Heading>
            <Heading
              level={HeadingLevel.l4}
              className="my-0 text-muted-foreground"
            >
              {notMooseContent.description}
            </Heading>
          </HalfWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
export const WhatIsMooseFor = () => {
  return (
    <>
      <Section className="mx-auto xl:max-w-screen-xl">
        <FullWidthContentContainer>
          <Heading>{mooseContent.title}</Heading>
        </FullWidthContentContainer>
        <Grid className="gap-y-5 justify-center">
          {mooseContent.usecases.map((usecase, index) => {
            return (
              <Fragment key={index}>
                <HalfWidthContentContainer
                  key={index}
                  className="flex flex-col xl:justify-start xl:order-4 p-5"
                >
                  <div className="relative aspect-square my-0 px-10">
                    <Suspense fallback={<div>Loading...</div>}>
                      <TemplateImg
                        srcDark={usecase.imageSrcDark}
                        srcLight={usecase.imageSrcLight}
                        alt={usecase.title}
                      />
                    </Suspense>
                  </div>
                  <div className="flex flex-col gap-5">
                    <Badge className="w-fit p-2" variant={"default"}>
                      {usecase.badge}
                    </Badge>
                    <Heading level={HeadingLevel.l2} className="my-0">
                      {usecase.title}
                    </Heading>
                    <Heading
                      level={HeadingLevel.l4}
                      className="my-0 text-muted-foreground xl:grow"
                    >
                      {usecase.description}
                    </Heading>
                  </div>
                </HalfWidthContentContainer>
              </Fragment>
            );
          })}
        </Grid>
      </Section>
    </>
  );
};
