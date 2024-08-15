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

import { Badge } from "@514labs/design-system-components/components";

export const WhatIsMooseFor = () => {
  return (
    <>
      <Section className="max-w-5xl mx-auto">
        <FullWidthContentContainer>
          <Heading
            level={HeadingLevel.l1}
            className="max-w-5xl justify-center align-center text-center mb-24 sm:text-5xl"
          >
            When to use Moose{" "}
          </Heading>
        </FullWidthContentContainer>
        <Grid className="justify-center">
          <HalfWidthContentContainer className="flex flex-col xl:justify-start xl:order-4 border rounded-3xl p-8 gap-5">
            <Badge className="w-fit p-2" variant={"outline"}>
              Moose + User-Facing App
            </Badge>
            <Heading level={HeadingLevel.l3} className="my-0">
              Data Intensive Features
            </Heading>
            <Heading
              level={HeadingLevel.l4}
              className="text-muted-foreground my-0"
            >
              Build an analytics backend to power real-time
              <span className="bg-[linear-gradient(150.33deg,_#641bff_-210.85%,_#1983ff_28.23%,_#ff2cc4_106.53%)] bg-clip-text text-transparent">
                {" "}
                leaderboards, charts, and metrics
              </span>{" "}
              in your products
            </Heading>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer className="flex flex-col xl:justify-start xl:order-4 border rounded-3xl p-8 gap-5">
            <Badge className="w-fit p-2" variant={"outline"}>
              Moose + Enterprise BI
            </Badge>
            <Heading level={HeadingLevel.l3} className="my-0">
              Enterprise Data Products
            </Heading>
            <Heading
              level={HeadingLevel.l4}
              className="text-muted-foreground my-0"
            >
              Pull data together from multiple sources and expose it to{" "}
              <span className="bg-[linear-gradient(150.33deg,_#641bff_-210.85%,_#1983ff_28.23%,_#ff2cc4_106.53%)] bg-clip-text text-transparent">
                {" "}
                BI platforms, AI/ML pipelines, and notebooks
              </span>{" "}
            </Heading>
          </HalfWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
