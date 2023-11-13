import { Metadata } from "next";
import { AnimatedHeading } from "./components/AnimatedHeading";
import { SectionGrid } from "./components/SectionGrid";
import { gsap } from "gsap";


import { AnimatedImage } from "./components/AnimatedImage";
import { ButtonStyle} from "./components/Button";
import { AnimatedDescription } from "./components/AnimatedDescription";
import { CodeBlockCTA } from "./components/CodeBlockCTA";


import heroImg from "../../public/woman/woman-0.webp"
import secondImg from "../../public/woman/woman-1.webp"
import iglooImg from "../../public/circles-mesh/image_mesh_4x.webp"
import thirdImg from "../../public/woman/woman-3.webp"
import fourthImg from "../../public/woman/woman-4.webp"
import { FooterSection } from "../sections/FooterSection";
import { AnimatedComponent } from "./components/AnimatedComponent";

export const metadata: Metadata = {
  title: "Igloo | The Data Platform for Developers",
  openGraph: {
    images: "../../public/open-graph/og_igloo_4x.webp"
  }
};


const fiveonefourSection = {
  heading: "build for the modern data stack",
  description: "Igloo is a batteries-included framework for building data-intensive applications using Typescript or Python, and SQL. It comes with a powerful CLI to help automate development tasks, an intuitive abstraction to help you build quickly, and a streamlined local development workflow.",
}

const secondSection = {
  heading: "decades of best practices",
  description: "We're building the frameworks, tools and infrastructure to help any developer build data-intensive applications quickly and easily.",
}

const middleSection = {
  heading: "modernized & open for all",
  description: "We're building the frameworks, tools and infrastructure to help any developer build data-intensive applications quickly and easily.",
}

const finalSection = {
  heading: "start building today",
  description: "Start building your data-intensive application today. Igloo is free to use and open source. If you'd like to contribute, check out our github or join our discord.",
}

const features = [
  {
    heading: "✦ Local First",
    description: "Build your application locally, and deploy to the cloud when you're ready. No need to configure a cloud environment to get started.",
  },
  {
    heading: "✦ Abstractions",
    description: "Igloos abstractions help you focus on building your end-to-end application without worrying about the underlying infrastructure.",
  },
  {
    heading: "✦ Intuitive Modeling",
    description: "We love working with Prisma in when building web apps. We've brought their modeling language to the modern data stack to help you create intuitive and readable data models.",
  },
  {
    heading: "✦ Zero Config",
    description: "Get up and running with your application in minutes. Igloo comes with a powerful CLI to help you automate development tasks. ",
  },
  {
    heading: "✦ Popular Languages",
    description: "Write your application in Typescript or Python, build your data models in Prisma and use SQL to interact with your OLAP databases",
  },
  {
    heading: "✦ Easy Testing",
    description: "Write tests for your application using the languages you love. Igloo supports your favorite frameworks in Typescript and Python ",
  },
  
];

const modernized = [
  {
    heading: "✦ Effortless Setup",
    description: "Get up and running with your application in minutes. Igloo comes with a powerful CLI to help you automate development tasks.",
  },
  
  {
    heading: "✦ Analytics Storage",
    description: "We use the latest generation of analytics storage to guarantee performance and a great local development experience. Native clickhouse support with DuckDB and Delta Lake coming soon.    ",
  },
  {
    heading: "✦ Fully Integrated Stack",
    description: "We've composed igloo from best-in-class data infrastructure to enable you to run your entire modern data stack on your local machine. No more configuring connections to start building.",
  },
  {
    heading: "✦ Best in-class Streaming",
    description: "We love working with Prisma in when building web apps. We've brought their modeling language to the modern data stack to help you create intuitive and readable data models.    ",
  },
  
];

export default function Home() {

  return (
    <div className="h-full relative">

      <SectionGrid className="mt-12 mb-24 2xl:mt-0" gapStyle="gap-y-12 xl:gap-y-36">
          <div className="flex flex-col px-10 w-full space-y-5 col-span-3 sm:col-span-12 2xl:col-span-6">
            <AnimatedHeading position={0} className="" content={fiveonefourSection.heading} size="display" />
            <AnimatedDescription position={1} className="" content={fiveonefourSection.description} />
            <CodeBlockCTA />
          </div>
          <div className="relative h-full w-full min-h-[80vw] sm:min-h-[50vw] col-span-3 sm:col-span-12 2xl:col-span-6">
            <AnimatedImage src={heroImg} position={2} alt="the crew" priority/>
          </div>
       </SectionGrid>

      {/* <div className="my-24 2xl:my-48">
        <AnimatedHeading position={1} className="px-10 w-full" content="our mission is to enable developers to build, deploy & scale data-intensive apps through igloo, an open-source data-intensive application framework." size="express" onScroll/>
      </div> */}
        <div className="flex flex-col w-full px-10 space-y-10 col-span-3 sm:col-span-12 2xl:col-span-6">
            <AnimatedHeading position={1} className="" content={secondSection.heading} size="display" />
          </div>


          <SectionGrid gapStyle="gap-10" itemPosition="start" className="px-10 space-y-0 lg:space-y-0 lg:flex-row 2xl:my-36 my-24">
            {features.map((feature, index) => {
              return (
                <div key={index} className="flex flex-col md:flex-row flex-1 col-span-3 sm:col-span-9 lg:col-span-6 xl:col-span-4">
                <div className="flex flex-col md:flex-1">
                    <div className="text-action-primary text-2xl">
                      <AnimatedHeading position={1} size="heading" content={feature.heading} onScroll />
                    </div>
                    <div className="text-typography-primary">
                      <AnimatedDescription position={2} content={feature.description} onScroll />
                    </div>
                  </div>
                </div>
              );
            })}
            </SectionGrid>

          <div className="flex flex-col w-full px-10 space-y-10 col-span-3 sm:col-span-12 2xl:col-span-6 my-36">
            <AnimatedHeading position={0} className="" content={middleSection.heading} size="display" onScroll />
          </div>
          <div className="relative h-full w-full min-h-[80vw] sm:min-h-[50vw] col-span-3 sm:col-span-12 2xl:col-span-6">
            <AnimatedImage src={thirdImg} position={0} alt="the crew" priority onScroll />
          </div>

          <SectionGrid gapStyle="gap-10" itemPosition="start" className="px-10 align-top space-y-0 lg:space-y-0 lg:flex-row 2xl:my-36 my-24">

          {modernized.map((feature, index) => {
            return (
              <div key={index} className="flex flex-col md:flex-row flex-1 col-span-3 sm:col-span-9 lg:col-span-6">
                <div className="flex flex-col md:flex-1">
                  <div className="text-action-primary text-2xl">
                    <AnimatedHeading position={1} size="heading" content={feature.heading} onScroll />
                  </div>
                  <div className="text-typography-primary">
                    <AnimatedDescription position={2} content={feature.description} onScroll />
                  </div>
                </div>
              </div>
            );
          })}
        </SectionGrid>

          <SectionGrid className="mt-12 mb-24 2xl:mt-0" gapStyle="gap-y-24 xl:gap-y-36">
          <div className="relative h-full w-full min-h-[80vw] sm:min-h-[50vw] col-span-3 sm:col-span-12 2xl:col-span-6">
            <AnimatedImage src={fourthImg} position={0} alt="the crew" onScroll />
          </div>
          <div className="flex flex-col px-10 w-full space-y-5 col-span-3 sm:col-span-12 2xl:col-span-6">
            <AnimatedHeading position={1} className="" content={finalSection.heading} size="display" />
            <AnimatedDescription position={2} className="" content={finalSection.description} />
            <CodeBlockCTA />
          </div>
       </SectionGrid>
       
      <FooterSection />
    </div>
    
  );
}
