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
        "Build an analytics backend to power real-time leaderboards, charts, and metrics in your OLTP apps",
      badge: "Moose + OLTP",
      imageSrcLight: "/images/diagrams/img-diagram-data-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-data-dark.svg",
    },
    {
      title: "Enterprise data products",
      description:
        "Build a data warehouse to serve BI tools, AI/ML pipelines, and data exploration notebooks",
      badge: "Moose + BI",
      imageSrcLight: "/images/diagrams/img-diagram-ent-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-ent-dark.svg",
    },
  ],
};

const notMooseContent = {
  title: "When not to use Moose",
  subtitle: "Transactional Apps",
  description:
    "If you’re building apps with only high transactional workloads and CRUD operations, you’ll be better served by the decades of great OLTP focused frameworks out there",
  badge: "OLTP",
  imageSrcLight: "/images/diagrams/img-diagram-standard-light.svg",
  imageSrcDark: "/images/diagrams/img-diagram-standard-dark.svg",
};

export const WhatIsntMooseFor = () => {
  return (
    <>
      <Section className="mx-auto xl:max-w-screen-xl border-t">
        <FullWidthContentContainer className="my-12 2xl:mb-20">
          <Heading>{notMooseContent.title}</Heading>
        </FullWidthContentContainer>
        <Grid className="items-center">
          <HalfWidthContentContainer>
            <div className="relative aspect-[2/1] my-10 p-5">
              <Suspense fallback={<div>Loading...</div>}>
                <TemplateImg
                  srcDark={notMooseContent.imageSrcDark}
                  srcLight={notMooseContent.imageSrcLight}
                  alt={notMooseContent.subtitle}
                />
              </Suspense>
            </div>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer className="flex flex-col gap-2 align-bottom">
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
        <FullWidthContentContainer className="mb-12 2xl:mb-20">
          <Heading>{mooseContent.title}</Heading>
        </FullWidthContentContainer>
        <Grid className="justify-center">
          {mooseContent.usecases.map((usecase, index) => {
            return (
              <Fragment key={index}>
                <HalfWidthContentContainer
                  key={index}
                  className="flex flex-col xl:justify-start xl:order-4"
                >
                  <div className="relative aspect-[2/1] my-10">
                    <Suspense fallback={<div>Loading...</div>}>
                      <TemplateImg
                        srcDark={usecase.imageSrcDark}
                        srcLight={usecase.imageSrcLight}
                        alt={usecase.title}
                      />
                    </Suspense>
                  </div>
                  <div className="flex flex-col gap-2">
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
