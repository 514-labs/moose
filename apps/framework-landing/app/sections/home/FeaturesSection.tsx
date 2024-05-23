import {
  Section,
  Grid,
  HalfWidthContentContainer,
} from "@514labs/design-system/components/containers";
import { Heading } from "@514labs/design-system/typography";
import React from "react";
import { FeatureGrid } from "./FeatureGrid";

export const featureContent = {
  title: "Everything you need to build data-driven experiences",
  features: [
    {
      title: "Data models",
      description:
        "Define data models in your language, Moose derives the infrastructure for you",
    },
    {
      title: "Flows",
      description:
        "Write simple functions to transform and augment your data on the fly",
    },
    {
      title: "Aggregations & metrics",
      description:
        "Pivot, filter and group your data for repeatable insights and metrics",
    },
    {
      title: "Migrations",
      description:
        "Moose helps manage migrations for your end-to-end data  stack",
    },
    {
      title: "Templates",
      description:
        "Get up and running in seconds with pre-built application templates",
    },
    {
      title: "Packaging",
      description:
        "Easily package your application for deployment in any environment",
    },
    {
      title: "UI Components (coming soon)",
      description:
        "Embed insightful react components in your framework of choice",
    },
    {
      title: "Connectors & SDKs (coming soon)",
      description:
        "Connectors and auto generated SDKs get data in and out of moose",
    },
    {
      title: "Orchestration (coming soon)",
      description:
        "Configurable orchestration to make sure data gets where it needs to go reliably",
    },
  ],
};

interface Content {
  title: string;
  features: { title: string; description: string }[];
}
const FeatureOverview = ({ content }: { content: Content }) => {
  return (
    <>
      <Section className="mx-auto xl:max-w-screen-xl">
        <Grid className="mb-12 2xl:mb-20">
          <HalfWidthContentContainer>
            <Heading> {content.title} </Heading>
          </HalfWidthContentContainer>
        </Grid>
        <FeatureGrid features={content.features} />
      </Section>
    </>
  );
};

export const FeaturesSection = () => (
  <FeatureOverview content={featureContent} />
);
