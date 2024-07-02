import {
  FullWidthContentContainer,
  Grid,
  ThirdWidthContentContainer,
  Section,
} from "@514labs/design-system-components/components/containers";

import {
  Heading,
  Text,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import React from "react";
import { Fragment, Suspense } from "react";

import { TemplateImg } from "../../sections/home/TemplateImg";
import { Badge } from "@514labs/design-system-components/components";

const content = {
  title: "Build big and small data features and products",
  description:
    "Moose helps you turn big and small data into features and products for your users and your peers",
  usecases: [
    {
      title: "Standard Apps",
      description: "COPY HERE",
      badge: "OLTP Frameworks",
      imageSrcLight: "/images/diagrams/img-diagram-standard-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-standard-dark.svg",
    },
    {
      title: "Data Intensive Features",
      description:
        "Extend your user facing applications with dynamic leaderboards, charts, and data feeds",
      badge: "OLTP Frameworks & Moose",
      imageSrcLight: "/images/diagrams/img-diagram-data-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-data-dark.svg",
    },
    {
      title: "Data as a Product",
      description:
        "Make it easy for others to derive value from the data your team owns",
      badge: "Moose",
      imageSrcLight: "/images/diagrams/img-diagram-ent-light.svg",
      imageSrcDark: "/images/diagrams/img-diagram-ent-dark.svg",
    },
  ],
};

export const WhatIsMooseFor = () => {
  return (
    <>
      <Section className="mx-auto xl:max-w-screen-xl">
        <Grid className="mb-12 2xl:mb-20">
          <FullWidthContentContainer>
            <Heading>{content.title}</Heading>
            <Heading level={HeadingLevel.l3} className="text-muted-foreground">
              {content.description}
            </Heading>
          </FullWidthContentContainer>
        </Grid>
      </Section>
      <Section className="w-full relative mx-auto xl:my-10 xl:max-w-screen-xl 2xl:my-0">
        <Grid className="gap-y-5 justify-center">
          {content.usecases.map((usecase, index) => {
            return (
              <Fragment key={index}>
                <ThirdWidthContentContainer
                  key={index}
                  className="flex flex-col xl:justify-start xl:order-4 border border-muted-foreground rounded-3xl p-5"
                >
                  <div className="relative aspect-square my-0">
                    <Suspense fallback={<div>Loading...</div>}>
                      <TemplateImg
                        srcDark={usecase.imageSrcDark}
                        srcLight={usecase.imageSrcLight}
                        alt={usecase.title}
                      />
                    </Suspense>
                  </div>

                  {/* <Text className="flex flex-row items-center justify-start border border-primary rounded-full px-5 py-2.5 w-fit">
                    {usecase.badge}
                  </Text>
                   */}
                  <Badge className="w-fit p-2" variant={"outline"}>
                    {usecase.badge}
                  </Badge>
                  <div className="my-5">
                    <Text className="my-0">{usecase.title}</Text>
                    <Text className="my-0 text-muted-foreground xl:grow">
                      {usecase.description}
                    </Text>
                  </div>
                </ThirdWidthContentContainer>
              </Fragment>
            );
          })}
        </Grid>
      </Section>
    </>
  );
};
