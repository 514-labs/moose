"use client";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
} from "@514labs/design-system-components/components";
import {
  Grid,
  HalfWidthContentContainer,
  Section,
  FullWidthContentContainer,
  ThirdWidthContentContainer,
  TwoThirdsWidthContentContainer,
} from "@514labs/design-system-components/components/containers";

import {
  Heading,
  Text,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import React, { useEffect, useRef, useState } from "react";

import { TrackableAccordionTrigger } from "../../trackable-components";
import Diagram from "../../spline";

const content = {
  title: "Build your features and products. Moose handles the stack.",
  description:
    "Moose automatically manages the infrastructure, so you can focus on innovating with your data",
  top: {
    title: "Analytics or User Facing Applications",
    description:
      "Deliver structured data and insights to user facing applications, AI/ML models, analyst notebooks, or enterprise BI software",
  },
  layers: [
    {
      title: "Data Application Logic",
      label: "TOP-LAYER",
      description:
        "Develop application logic for modeling, processing, aggregating, and consuming data using Moose primitives",
      details: [
        {
          title: "Models",
          description: "Define a schema for raw data sources to ingest",
        },
        {
          title: "Flows",
          description: "Write functions to augment & transform data",
        },
        {
          title: "Blocks",
          description: "Create views to slice, aggregate, and join tables",
        },
        {
          title: "APIs",
          description: "Fetch and serve insights and metrics to your apps",
        },
      ],
    },
    {
      title: "Moose Defined Infrastructure",
      description:
        "Moose automatically configures and manages the systems needed for data processing, storage, and access, based on your data application logic",
      layer: "MIDDLE-LAYER",
      details: [
        {
          title: "Ingestion Endpoints",
          description: "Routes incoming data to appropriate destination",
        },
        {
          title: "Topics",
          description: "Buffers data to avoid loss during peak load times",
        },
        {
          title: "Processes",
          description: "Executes transformation functions",
        },
        {
          title: "Tables",
          description:
            "Stores structured data for efficient retrieval and analytics",
        },
        {
          title: "Views",
          description: "Stores aggregated data for reuse & performance",
        },
        {
          title: "Consumption Endpoints",
          description: "Executes route handlers to serve data to client apps",
        },
      ],
    },

    {
      title: "Foundational Infrastructure",
      description:
        "Moose uses modern, open-source solutions in an industry-standard data stack, managing the connections to ensure reliable data transmission across systems",
      layer: "BOTTOM-LAYER",
      details: [
        {
          title: "Webserver",
          description:
            "Rust-based ingest and language specific consumption servers balance speed and customizability",
        },
        {
          title: "Processing",
          description:
            "Async runtime for real-time data transformation and workflow orchestration",
        },
        {
          title: "Streaming",
          description:
            "Fully integrated streaming for managing variable data volumes. Redpanda supported (more coming soon)",
        },
        {
          title: "Storage",
          description:
            "Best-in-class OLAP storage that’s easy to use and configure,  Clickhouse supported (more coming soon)",
        },
      ],
    },
  ],
  bottom: {
    title: "Raw Data Source",
    description:
      "Ingest data from applications, databases, blob storage, IoT devices, and more",
  },
};

const MooseLayersAccordion = ({ spline }: { spline: any }) => {
  const [expanded, setExpanded] = useState<boolean>(false);
  return (
    <div>
      <Accordion
        type="single"
        collapsible
        onValueChange={(val) => {
          const outerWrap = spline.current?.findObjectByName("OUTER-WRAP");
          if (!val) {
            setExpanded(false);
            outerWrap?.emitEventReverse("mouseDown");
          }
          if (!expanded) {
            setExpanded(true);
            outerWrap?.emitEvent("mouseDown");
          }
        }}
      >
        {content.layers.map((layer, index) => {
          return (
            <AccordionItem
              key={index}
              value={`item-${index}`}
              onClick={() => {}}
            >
              <TrackableAccordionTrigger
                name="Moose Layer Accordion"
                subject={layer.title}
                className="hover:bg-muted-foreground/20 h"
              >
                <Text className="my-0">{layer.title}</Text>
              </TrackableAccordionTrigger>
              <AccordionContent>
                <div className="flex flex-col text-start justify-start gap-5">
                  <FullWidthContentContainer>
                    <Text className="my-0 text-muted-foreground">
                      {layer.description}
                    </Text>
                  </FullWidthContentContainer>
                  <Grid className="gap-10">
                    {layer.details.map((detail, index) => {
                      return (
                        <HalfWidthContentContainer
                          className="w-full flex flex-col items-center justify-stretch text-left border border-muted-foreground rounded-3xl p-5"
                          key={index}
                        >
                          <Text className="my-0 justify-start self-start">
                            {detail.title}
                          </Text>
                          <Text className="my-0 text-muted-foreground self-start">
                            {detail.description}
                          </Text>
                        </HalfWidthContentContainer>
                      );
                    })}
                  </Grid>
                </div>
              </AccordionContent>
            </AccordionItem>
          );
        })}
      </Accordion>
    </div>
  );
};

export const WhatIsMoose = () => {
  const spline = useRef();
  const [splineHeight, setSplineHeight] = useState(0);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!ref.current) {
      return;
    }
    const resizeObserver = new ResizeObserver(() => {
      if (ref.current && ref.current.offsetHeight !== splineHeight) {
        setSplineHeight(ref.current.offsetHeight);
      }
    });
    resizeObserver.observe(ref.current);
    return function cleanup() {
      resizeObserver.disconnect();
    };
  }, [ref.current]);

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
      <div></div>
      <Section className="w-full relative mx-auto xl:my-10 xl:max-w-screen-xl 2xl:my-0">
        <Grid>
          <ThirdWidthContentContainer>
            <Diagram height={splineHeight} spline={spline} />
          </ThirdWidthContentContainer>

          <TwoThirdsWidthContentContainer className="flex flex-col xl:justify-start gap-5 h-fit">
            <div ref={ref}>
              <FullWidthContentContainer className="px-4">
                <Text className="my-0">{content.top.title}</Text>
                <Text className="my-0 text-muted-foreground">
                  {content.bottom.description}
                </Text>
              </FullWidthContentContainer>
              <MooseLayersAccordion spline={spline} />
              <FullWidthContentContainer className="px-4">
                <Text className="my-0">{content.bottom.title}</Text>
                <Text className="my-0 text-muted-foreground">
                  {content.bottom.description}
                </Text>
              </FullWidthContentContainer>
            </div>
          </TwoThirdsWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
