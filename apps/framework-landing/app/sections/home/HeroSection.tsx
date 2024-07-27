import { CTABar } from "../../page";
import {
  Section,
  FullWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import {
  Heading,
  Display,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import { TrackCtaButton } from "../../trackable-components";
import React, { Fragment } from "react";
import Link from "next/link";
import {
  HardDriveDownload,
  RectangleEllipsis,
  Table,
  Code,
  Box,
  HardDriveUpload,
} from "lucide-react";
import { cn } from "@514labs/design-system-components/utils";

const FeatureHighlightCard = ({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) => {
  return (
    <div className="bg-gradient-to-b from-muted-foreground to-muted rounded-[30px] p-[2px] grow">
      <div
        className={cn("bg-background rounded-[28px] p-5 text-xs", className)}
      >
        {children}
      </div>
    </div>
  );
};

const styleMapperRoute = (index: number) => {
  if (
    index === 2 ||
    // index === 3 ||
    index === 20 ||
    // index === 21 ||
    index === 38
    // index === 39
  ) {
    return "border-b";
  }

  if (index === 3 || index === 21 || index === 39) {
    return " border-b overflow-hidden relative -left-[2px]";
  }

  if (
    index === 8 ||
    // index === 9 ||
    index === 26 ||
    // index === 27 ||
    index === 44
    // index === 45
  ) {
    return "border-t";
  }
  if (index === 9 || index === 27 || index === 45) {
    return "border-t overflow-hidden relative -left-[2px]";
  }

  if (
    index === 16 ||
    index === 30 ||
    index === 20 ||
    index === 21 ||
    index === 38 ||
    index === 39
  ) {
    return "border-r";
  }
  if (
    index === 17 ||
    index === 31 ||
    index === 26 ||
    index === 27 ||
    index === 44 ||
    index === 45
  ) {
    return "border-l";
  }

  return "";
};

const ValuePropHeroSection = () => {
  return (
    <section className="max-w-5xl mx-auto sm:my-64 mt-0 pt-0">
      <div className="flex lg:flex-row flex-col justify-between items-stretch gap-5">
        {/* Without framework card */}
        <div className="border p-5 space-y-5 rounded-2xl lg:w-1/2 w-full">
          {/* Titles */}
          <div className="space-y-2">
            <div className="text-3xl">With our framework</div>
            <div className="text-xl text-muted-foreground w-96">
              Get bogged down in boiler plate and integrate, scale and manage
              infra
            </div>
          </div>
          {/* Component grid */}
          <div className="relative p-12 aspect-square items-center flex">
            <div className="relative w-full">
              {/* Front grid */}
              <div className="grid grid-cols-2 gap-5 h-full z-10 relative">
                <div>
                  <FeatureHighlightCard className="flex flex-row justify-center">
                    <HardDriveDownload />
                  </FeatureHighlightCard>
                </div>
                <div>
                  <FeatureHighlightCard className="flex flex-row justify-center">
                    <RectangleEllipsis />
                  </FeatureHighlightCard>
                </div>
                <div>
                  <FeatureHighlightCard className="flex flex-row justify-center">
                    <Table />
                  </FeatureHighlightCard>
                </div>
                <div>
                  <FeatureHighlightCard className="flex flex-row justify-center">
                    <Code />
                  </FeatureHighlightCard>
                </div>
                <div>
                  <FeatureHighlightCard className="flex flex-row justify-center">
                    <Box />
                  </FeatureHighlightCard>
                </div>
                <div>
                  <FeatureHighlightCard className="flex flex-row justify-center">
                    <HardDriveUpload />
                  </FeatureHighlightCard>
                </div>
              </div>

              {/* Back grid */}
              <div className="grid grid-cols-6 absolute w-full h-full top-0 grid-rows-8 z-0 bg-gradient-to-l from-[#373FFF] to-[#B800C8] ">
                {/* programatically generate 6*8 divs */}
                {[...Array(6 * 8)].map((_, idx) => (
                  <div
                    key={idx}
                    className={cn(
                      "bg-background border-dashed bg-clip-padding border-background",
                      styleMapperRoute(idx),
                    )}
                  ></div>
                ))}
              </div>
            </div>
          </div>
        </div>
        <div className="w-[20px]"></div>

        {/* With framework card */}
        <div className="border p-5 space-y-5 rounded-2xl overflow-hidden lg:w-1/2">
          {/* Titles */}
          <div className="space-y-2 relative z-10">
            <div className="text-3xl">With our framework</div>
            <div className="text-xl text-muted-foreground w-96">
              Let your teams focus on building. Let us handle the grunt work
            </div>
          </div>
          {/* Moose */}
          {/* Component grid */}
          <div className="relative p-12 aspect-square flex items-center">
            <div className="relative w-full">
              {/* Front grid */}
              <div className="flex gap-5 h-full z-10 relative flex-row items-center justify-center">
                <div className=" relative">
                  {/* pins */}
                  <div className="h-full w-2 top-0 -left-1 absolute z-20 flex flex-col justify-center space-y-4">
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                  </div>
                  <div className="h-full w-2 top-0 -right-2 absolute z-20 flex flex-col justify-center space-y-4">
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                  </div>
                  <div className="h-2 w-full -top-1 left-0 absolute z-20 flex flex-row justify-center space-x-4">
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                  </div>
                  <div className="h-2 w-full -bottom-2 left-0 absolute z-20 flex flex-row justify-center space-x-4">
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                    <div className="w-1 h-1 rounded-full bg-gradient-to-tr from-[#373FFF] to-[#B800C8]" />
                  </div>
                  <div className="bg-gradient-to-b from-muted-foreground to-muted rounded-2xl p-[2px] grow">
                    <div className="bg-background rounded-2xl text-xl h-48 w-48 flex flex-col items-center justify-center relative overflow-hidden">
                      <div className="-mt-1 z-10 relative bg-transparent">
                        moose
                        <div></div>
                        <div></div>
                        <div></div>
                      </div>
                      <div className="absolute -top-1/2 -right-1/2 z-0 w-60 h-60 opacity-80 bg-gradient-to-b from-muted-foreground/50 to-background rounded-3xl blur-2xl" />
                    </div>
                  </div>
                </div>
              </div>

              {/* Back grid */}
              <div className="absolute -top-[20px] right-[20px] z-0 w-48 h-48 bg-gradient-to-b from-[#373FFF] to-background rounded-3xl blur-[100px]" />
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

export const HeroSection = () => {
  const content = {
    tagLine: "Prototype & scale data-intensive applications without the hassle",
    description:
      "Moose is an open source developer framework for your data & analytics stack",
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
        label: "View Docs ",
        variant: "outline",
      },
    ],
  };

  return (
    <Fragment>
      <Section className="2xl:max-w-6xl mx-auto flex flex-col items-center px-5 mt-16 sm:my-64">
        <FullWidthContentContainer className="pt-0 items-center flex flex-col gap-5 mb-0">
          <div>
            <Display className="my-0 text-center">{content.tagLine} </Display>

            <Heading
              level={HeadingLevel.l3}
              className="text-muted-foreground text-center"
            >
              {" "}
              {content.description}{" "}
            </Heading>
          </div>
          <CTABar className="mb-10 align-center justify-center">
            {content.ctas.map((cta, index) => (
              <Link key={index} href={cta.href}>
                <TrackCtaButton
                  name={`Hero CTA ${cta.label}`}
                  subject={content.tagLine}
                  targetUrl={cta.href}
                  variant={cta.variant as "default" | "outline"}
                >
                  {cta.label}
                </TrackCtaButton>
              </Link>
            ))}
          </CTABar>
        </FullWidthContentContainer>
      </Section>
      <ValuePropHeroSection />
    </Fragment>
  );
};
