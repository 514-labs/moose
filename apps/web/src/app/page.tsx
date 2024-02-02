import { Metadata } from "next";
import { AnimatedHeading } from "./components/AnimatedHeading";
import { SectionGrid } from "./components/SectionGrid";
import { gsap } from "gsap";

import { AnimatedImage } from "./components/AnimatedImage";
import { Button } from "./components/Button";
import { AnimatedDescription } from "./components/AnimatedDescription";

import heroImg from "../../public/bg-image-man/bg-image-hero_2x.webp";
import middleImg from "../../public/bg-image-computer/bg-image-computer_2x.webp";
import footerImg from "../../public/bg-image-moose/bg-image-moose_2x.webp";
import { FooterSection } from "./sections//home/FooterSection";
import { AnimatedComponent } from "./components/AnimatedComponent";
import { CodeBlockCTA } from "./components/CodeBlockCTA";

export const metadata: Metadata = {
  title: "Moose.js | Build for the modern data stack",
  openGraph: {
    images: "/open-graph/og_moose_4x.png",
  },
};

const frameworkSection = {
  heading: "start building today",
  description:
    "Start building your data-intensive application today. Moose.js is free to use and open source. If you'd like to contribute, check out our github or join our Slack.",
};

const fiveonefourSection = {
  heading: "build for the modern data stack",
  description:
    "Moose.js is a batteries-included framework for building data-intensive applications using Typescript or Python, and SQL. It comes with a powerful CLI to help automate development tasks, an intuitive abstraction to help you build quickly, and a streamlined local development workflow..",
};

const practices = [
  {
    heading: "Local first",
    description:
      "Build your application locally, and deploy to the cloud when you're ready. No need to configure a cloud environment to get started.",
  },
  {
    heading: "Best-in-class Streaming",
    description:
      "A highly performant and scalable data capture stack that scales with your volumes and is able to run locally, supporting Rust and Red Panda.",
  },
  {
    heading: "Popular Languages",
    description:
      "Write your application in Typescript or Python, build your data models in Prisma and use SQL to interact with your OLAP database. ",
  },
  {
    heading: "Test Efficiently",
    description:
      "Write tests for your application using the languages you love. Moose supports your favorite frameworks in Typescript and Python.",
  },
  {
    heading: "Effortless Setup",
    description:
      "Get up and running with your application in minutes. Moose comes with a powerful CLI to help you automate development tasks.",
  },
  {
    heading: "Simplified Abstractions",
    description:
      "Moose abstractions help you focus on building your end-to-end application without worrying about the underlying infrastructure.",
  },
];

// Some comment

const features = [
  {
    heading: "full data stack",
    description:
      "We've composed moose from best-in-class data infrastructure to enable you to run your entire modern data stack on your local machine. No more configuring connections to start building. ",
  },
  {
    heading: "data modeling",
    description:
      "We love working with Prisma in when building web apps. We've brought their modeling language to the modern data stack to help you create intuitive and readable data models. ",
  },
  {
    heading: "fast analytics",
    description:
      "We use the latest generation of analytics storage to guarantee performance and a great local experience. Native clickhouse support with DuckDB and Delta Lake coming soon. ",
  },
];

export default function Home() {
  return (
    <div className="h-full relative">
      <SectionGrid
        className="py-36 pb-0 sm:pt-24 2xl:pt-20"
        gapStyle="gap-y-36"
      >
        <div className="flex flex-col px-10 w-full space-y-5 col-span-3 sm:col-span-12 2xl:col-span-6">
          <AnimatedHeading
            position={0}
            className="2xl:text-8xl"
            content={fiveonefourSection.heading}
            size="display"
          />
          <AnimatedDescription
            position={0.75}
            className=""
            content={fiveonefourSection.description}
          />
          <div>
            <CodeBlockCTA />
          </div>
        </div>
        <div className="relative h-full w-full min-h-[80vw] sm:min-h-[50vw] col-span-3 sm:col-span-12 2xl:col-span-6">
          <AnimatedImage src={heroImg} position={0.5} alt="the crew" />
        </div>
      </SectionGrid>

      <div className="py-24">
        <AnimatedHeading
          className="px-10 w-full 2xl:text-8xl"
          content="decades of best practices"
          size="display"
          position={0.5}
          onScroll
        />
      </div>

      <SectionGrid
        gapStyle="gap-10"
        itemPosition="start"
        className="px-10 lg:space-y-0 lg:flex-row md:pb-12"
      >
        {practices.map((feature, index) => {
          return (
            <div
              key={index}
              className="flex flex-col md:flex-row flex-1 col-span-3 sm:col-span-6 lg:col-span-4"
            >
              <div className="flex flex-col md:flex-1">
                <div className="text-typography-primary">
                  <AnimatedHeading
                    className="text-black"
                    position={0.8}
                    size="heading"
                    content={feature.heading}
                    onScroll
                  />
                </div>
                <div className="text-typography-secondary">
                  <AnimatedDescription
                    position={1}
                    content={feature.description}
                    onScroll
                  />
                </div>
              </div>
            </div>
          );
        })}
      </SectionGrid>

      {/* <div className="py-20">
        <AnimatedHeading position={0.5} className="px-10 w-ful text-black" content="modernized & open for all" size="display" onScroll/>
      </div>

      <div className="relative h-full w-full min-h-[80vw] sm:min-h-[50vw] w-full space-y-5 mt-12 mb-24 2xl:mt-0">
          <AnimatedImage src={middleImg} position={0} alt="the crew" priority onScroll/>
        </div> */}

      <SectionGrid className="py-24 pb-0 2xl:pt-20" gapStyle="gap-y-36">
        <div className="flex flex-col px-10 w-full space-y-5 col-span-3 sm:col-span-12 2xl:col-span-6">
          <AnimatedHeading
            position={0.5}
            className="2xl:text-8xl"
            content="modernized & open for all"
            size="display"
            onScroll
          />
        </div>
        <div className="relative h-full w-full min-h-[80vw] sm:min-h-[50vw] col-span-3 sm:col-span-12 2xl:col-span-6">
          <AnimatedImage
            src={middleImg}
            position={0.25}
            alt="the crew"
            onScroll
          />
        </div>
      </SectionGrid>

      <SectionGrid
        gapStyle="gap-10"
        itemPosition="start"
        className="px-10 p-24 lg:space-y-0 lg:flex-row"
      >
        {features.map((feature, index) => {
          return (
            <div
              key={index}
              className="flex flex-col md:flex-row flex-1 col-span-3 sm:col-span-4 lg:col-span-4"
            >
              <div className="flex flex-col md:flex-1">
                <div className="text-typography-primary">
                  <AnimatedHeading
                    className="text-black"
                    position={0.75}
                    content={feature.heading}
                    onScroll
                    size="heading"
                  />
                </div>
                <div className="text-typography-primary">
                  <AnimatedDescription
                    position={1}
                    content={feature.description}
                    onScroll
                  />
                </div>
              </div>
            </div>
          );
        })}
      </SectionGrid>

      <SectionGrid gapStyle="gap-y-24 pb-0">
        <div className="relative h-full w-full min-h-[80vw] sm:min-h-[50vw] col-span-3 sm:col-span-12 xl:col-span-6">
          <AnimatedImage
            src={footerImg}
            position={0.25}
            alt="the crew"
            onScroll
          />
        </div>
        <div className="flex flex-col px-10 w-full space-y-5 col-span-3 sm:col-span-12 xl:col-span-6">
          <AnimatedHeading
            className="text-black 2xl:text-8xl"
            position={0.75}
            content={frameworkSection.heading}
            onScroll
            size="display"
          />
          <AnimatedDescription
            position={1}
            className=""
            content={frameworkSection.description}
            onScroll
          />
          <div>
            <CodeBlockCTA />
          </div>
        </div>
      </SectionGrid>
      <FooterSection />
    </div>
  );
}
