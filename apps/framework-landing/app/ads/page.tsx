import { ReactNode } from "react";

import { VariantProps } from "class-variance-authority";
import { FooterSection } from "../sections/FooterSection";
import { WhyMooseSection } from "../sections/home/WhyMooseSection";

import { SecondaryCTASection } from "../sections/home/SecondaryCTASection";
import { cn } from "@514labs/design-system/utils";

import { Button, buttonVariants } from "@514labs/design-system/components";

import {
  Heading,
  Text,
  Display,
  HeadingLevel,
} from "@514labs/design-system/typography";
import React from "react";
import {
  Section,
  Grid,
  HalfWidthContentContainer,
  FullWidthContentContainer,
  ThirdWidthContentContainer,
} from "@514labs/design-system/components/containers";
import Link from "next/link";
import { TrackCtaButton } from "../trackable-components";

export const CTAText = ({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <div
      className={cn(
        "text-center md:text-start text-primary text-4xl bg-muted rounded-md py-5 px-10 text-nowrap",
        className,
      )}
    >
      {children}
    </div>
  );
};

export interface CTAButtonProps extends VariantProps<typeof buttonVariants> {
  className?: string;
  children: ReactNode;
  onClick?: () => void;
}

export const CTAButton = ({
  className,
  children,
  variant,
  onClick,
}: CTAButtonProps) => {
  return (
    <Button
      size={"lg"}
      variant={variant}
      className={cn(
        "h-full font-normal border-primary w-full sm:w-auto px-10 py-0 rounded-xl",
        className,
      )}
      onClick={onClick}
    >
      <Text
        className={cn(
          variant === "outline" ? "text-primary" : "text-primary-foreground",
        )}
      >
        {children}
      </Text>
    </Button>
  );
};

export const CTABar = ({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <div className={cn("flex flex-col md:flex-row gap-5", className)}>
      {children}
    </div>
  );
};

export default function Home() {
  const heroContent = {
    tagLine: "A mixpanel alternative",
    description:
      "An open source developer framework for your data & analytics stack",
    ctas: [
      {
        href: "https://docs.moosejs.com/getting-started/new-project",
        action: "cta-early-access",
        label: "Get Started",
        variant: "default",
      },
      {
        href: "https://docs.moosejs.com/",
        action: "cta-early-access",
        label: "Learn More",
        variant: "outline",
      },
    ],
  };

  const featuresContent = {
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
          "Moose helps manage migrations for your end-to-end data stack",
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

  return (
    <main>
      <Section className="w-full relative mx-auto xl:max-w-screen-xl pb-10">
        <Grid>
          <HalfWidthContentContainer className="pt-0">
            <div>
              {/* <Heading> {heroContent.tagLine} </Heading> */}
              <Display className="my-0">{heroContent.tagLine} </Display>

              <Heading
                level={HeadingLevel.l2}
                className="text-muted-foreground"
              >
                {heroContent.description}
              </Heading>
            </div>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer className="pt-0">
            <CTABar className="mb-10">
              {heroContent.ctas.map((cta, index) => (
                <Link key={index} href={cta.href}>
                  <TrackCtaButton
                    name={cta.label}
                    subject={cta.label}
                    variant={cta.variant as "default" | "outline"}
                  >
                    {cta.label}
                  </TrackCtaButton>
                </Link>
              ))}
            </CTABar>
          </HalfWidthContentContainer>
        </Grid>
      </Section>
      <Section className="mx-auto xl:max-w-screen-xl">
        <Grid>
          {featuresContent.features.map((feature, index) => {
            return (
              <ThirdWidthContentContainer key={index}>
                <Text className="my-0">{feature.title}</Text>
                <Text className="my-0 text-muted-foreground">
                  {feature.description}
                </Text>
              </ThirdWidthContentContainer>
            );
          })}
        </Grid>
      </Section>
      <FooterSection />
    </main>
  );
}
