import { Metadata } from "next";
import { AnimatedHeading } from "./components/AnimatedHeading";
import { SectionGrid } from "./components/SectionGrid";

import { AnimatedImage } from "./components/AnimatedImage";
import { AnimatedDescription } from "./components/AnimatedDescription";

import heroImg from "../../public/bg-image-man/bg-image-hero_2x.webp";
import middleImg from "../../public/bg-image-computer/bg-image-computer_2x.webp";
import footerImg from "../../public/bg-image-moose/bg-image-moose_2x.webp";

import img1 from "../../public/carousel/img-1.png";
import img2 from "../../public/carousel/img-2.png";
import img3 from "../../public/carousel/img-3.png";
import img4 from "../../public/carousel/img-4.png";

import { FooterSection } from "./sections//home/FooterSection";
import { CodeBlockCTA } from "./components/CodeBlockCTA";
import { sendServerEvent } from "./events/sendServerEvent";

import Image, { StaticImageData } from "next/image";
import ProductAnalyticsTemplate from "./components/product-analytics-template";

export const metadata: Metadata = {
  title: "Moose.js | Build for the modern data stack",
  openGraph: {
    images: "/open-graph/og_moose_4x.png",
  },
};

const mooseHeroSection = {
  label: "For all developers",
  heading: "Delightful & Insightful",
  description: "The developer framework for your data & analytics stack",
};

const frameworkSection = {
  heading: "start building today",
  description:
    "Start building your data-intensive application today. Moose.js is free to use and open source. If you'd like to contribute, check out our github or join our Slack.",
};

const whyMoose = [
  {
    heading: "Your data stack, unified",
    description:
      "Jobs, pipelines, streams, data models, tables, views, schemas, APIs, and SDKs -- no more coordinating a tangled web of individual components. With a framework-based approach, each component is automatically integrated, so your data stack is easier to manage and more resilient to change.",
  },
  {
    heading: "Developers, delighted",
    description:
      "Moose simplifies the way you interact with you data and analytics stack to let you build rich data-intensive experiences using the languages you know,  the application frameworks you love and the tools you leverage in your day-to-day workflow.",
  },
  {
    heading: "Best practices, unlocked",
    description:
      "Moose’s approach brings the standard software development workflow to your data stack, in your language, enabling best practices like: local development, rapid iteration, CI/CD, automated testing, version control, change management, code collaboration, DevOps, etc.",
  },
];

const features = [
  {
    heading: "Git-based",
    description:
      "Git-based workflow enables software development best practices like version control and CI/CD",
  },
  {
    heading: "Local dev server",
    description:
      "Run your whole stack locally: see and test the impact of changes in real time as you edit code - just like developing a web app frontend",
  },
  {
    heading: "End-to-end typed",
    description:
      "Moose types all your components from ingest through consumption, so you catch issues at build time, instead of in production",
  },
  {
    heading: "Built-in testing",
    description:
      "Moose has a built in testing framework. Manage sample data and automatically test data pipelines as you’re developing",
  },
  {
    heading: "Magical change management",
    description:
      "Moose automatically manages schema evolution across versions, so you don’t break upstream/downstream dependencies",
  },
  {
    heading: "Data product APIs",
    description:
      "Automatic ingest and consumption API endpoints for your data products, with pre-built SDKs in the language of your choice (javascript, python, java, rust, etc).",
  },
];

export default async function Home() {
  const ip_obj = await sendServerEvent("page_view", { page: "home" });

  return (
    <div className="h-full relative">
      <SectionGrid
        className="py-36 pb-0 sm:pt-24 2xl:pt-48"
        gapStyle="gap-y-36"
      >
        <div className="col-span-6" />
        <div className="flex flex-col px-10 w-full space-y-10 col-span-3 sm:col-span-12 2xl:col-span-6">
          <AnimatedHeading
            position={0}
            className="capitalize font-light"
            content={mooseHeroSection.label}
            size="express"
          />
          <AnimatedHeading
            position={0}
            className="2xl:text-9xl"
            content={mooseHeroSection.heading}
            size="display"
          />
          <AnimatedHeading
            position={0.75}
            className="capitalize font-light"
            content={mooseHeroSection.description}
            size="express"
          />

          <CodeBlockCTA identifier={ip_obj.ip} />
        </div>
      </SectionGrid>
      <div className="flex flex-row space-x-8 py-20">
        <Image src={img1} alt="the crew" priority />
        <Image src={img2} alt="the crew" priority />
        <Image src={img3} alt="the crew" priority />
        <Image src={img4} alt="the crew" priority />
      </div>

      <div className="py-24">
        <AnimatedHeading
          className="px-10 w-full 2xl:text-8xl"
          content="Why moose?"
          size="display"
          position={0.5}
          onScroll
        />
      </div>

      {whyMoose.map((why, index) => {
        return (
          <SectionGrid
            key={index}
            gapStyle="gap-10"
            itemPosition="start"
            className="px-10 lg:space-y-0 lg:flex-row md:pb-12"
          >
            <div className="text-typography-primary col-span-6">
              <AnimatedHeading
                className="text-black"
                position={0.8}
                size="display-md"
                content={why.heading}
                onScroll
              />
            </div>
            <div className="text-typography-secondary col-span-6">
              <AnimatedDescription
                position={1}
                content={why.description}
                onScroll
                className="text-4xl font-light"
              />
            </div>
          </SectionGrid>
        );
      })}

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

      <div className="py-24">
        {features.map((feature, index) => {
          return (
            <SectionGrid
              key={index}
              gapStyle="gap-10"
              itemPosition="start"
              className="px-10 lg:space-y-0 lg:flex-row md:pb-12"
            >
              <div className="text-typography-primary col-span-6">
                <AnimatedHeading
                  className="text-black"
                  position={0.8}
                  size="display-md"
                  content={feature.heading}
                  onScroll
                />
              </div>
              <div className="text-typography-secondary col-span-6">
                <AnimatedDescription
                  position={0.8}
                  content={feature.description}
                  onScroll
                  className="text-4xl font-light"
                />
              </div>
            </SectionGrid>
          );
        })}
      </div>

      <div>
        <SectionGrid
          gapStyle="px-10 lg:space-y-0 lg:flex-row md:pb-12"
          itemPosition="start"
        >
          <div className="col-span-6">
            <AnimatedHeading
              position={0.5}
              content="Product analytics"
              size="display-md"
              onScroll
            />
          </div>
          <div className="col-span-6">
            <ProductAnalyticsTemplate />
          </div>
        </SectionGrid>
      </div>

      <div>
        <SectionGrid
          gapStyle="px-10 lg:space-y-0 lg:flex-row md:pb-12"
          itemPosition="start"
        >
          <div className="col-span-6">
            <AnimatedHeading
              position={0.5}
              content="Product analytics"
              size="display-md"
              onScroll
            />
          </div>
          <div className="col-span-6">
            <ProductAnalyticsTemplate />
          </div>
        </SectionGrid>
      </div>

      <div>
        <SectionGrid
          gapStyle="px-10 lg:space-y-0 lg:flex-row md:pb-12"
          itemPosition="start"
        >
          <div className="col-span-6">
            <AnimatedHeading
              position={0.5}
              content="Product analytics"
              size="display-md"
              onScroll
            />
          </div>
          <div className="col-span-6">
            <ProductAnalyticsTemplate />
          </div>
        </SectionGrid>
      </div>

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
            <CodeBlockCTA identifier={ip_obj.ip} />
          </div>
        </div>
      </SectionGrid>
      <FooterSection />
    </div>
  );
}
