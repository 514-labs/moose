import { Metadata } from "next";
import React from "react";
import { BackgroundImage } from "./BackgroundImage";
import { CodeBlockCTA } from "./CodeBlockCTA";
import { RightsComponent } from "./RightsComponent";
import { LogoComponent } from "./LogoComponent";
import Image from "next/image";

export const metadata: Metadata = {
  title: "Igloo | Build Data-intensive apps with ease",
  openGraph: {
    images: "/og_igloo_image_person_012_2x.webp"
  }
};


const features = [
  {
    heading: "Local first",
    description: "Build your application locally, and deploy to the cloud when you&apos;re ready. No need to configure a cloud environment to get started.",
  },
  {
    heading: "Languages you love",
    description: "Write your application in Typescript or Python, build your data models in Prisma and use SQL to interact with your OLAP database.",
  },
  // {
  //   heading: "Realtime ready",
  //   description: "Build realtime data-intensive applications with ease. Igloo comes with realtime support out of the box. ",
  // },
  {
    heading: "Easy to test",
    description: "Write tests for your application using the languages you love. Igloo supports your favorite frameworks in Typescript and Python.",
  },
  {
    heading: "Zero configuration",
    description: "Get up and running with your application in minutes. Igloo comes with a powerful CLI to help you automate development tasks.",
  },
  {
    heading: "Abstracted complexity",
    description: "Igloos abstractions help you focus on building your end-to-end application without worrying about the underlying infrastructure.",
  }
]


const stack = [
  {
    "name": "End-to-end on your laptop",
    "description": "We&apos;ve composed igloo from best-in-class data infrastructure to enable you to run your entire stack on your local machine. No more configuring connections to start building.",
  },
  {
    "name": "Best-in-class streaming",
    "description": "We&apos;ve created a highly performant and scalable data capture stack that scales with your data volumes and is lightweight enough to run locally. Rust ingestion points & native support for Redpanda.",
  },
  {
    "name": "Modern analyics storage",
    "description": "We use the latest generation of analytics storage to guarantee performance and a great local development experience. Native clickhouse support with DuckDB and Delta Lake coming soon.",
  },
  {
    "name": "Intuitive data modeling",
    "description": "We love working with Prisma in when building web apps. We&apos;ve brought their modeling language to the modern data stack to help you create intuitive and readable data models.",
  },
]


export default function Home() {
  return (
    <div className="h-full">
      <div className=" pt-32 w-screen flex grow-1 flex-col md:pt-16 md:h-full">
        <div className="h-full flex flex-col md:flex-row flex-grow md:justify-center md:items-center">
          <div className="text-white flex-col px-5 md:flex-1 ">
            <div className="px-5 text-5xl sm:text-6xl 2xl:text-9xl">
              the application framework for the modern data stack
            </div>
            <div className="flex flex-col grow md:flex-1 p-5 space-y-5">
              <div className="text-typography-primary my-3">
                Igloo is a batteries-included framework for building data-intensive applications using typescript & SQL. It comes with a powerful CLI to help automate development tasks, an intuitive abstraction to help you build quickly, and a streamlined local development workflow.
              </div>
              <div>
                <CodeBlockCTA />
              </div>
            </div>
          </div>
          <div className="flex flex-auto md:flex-1 flex-row md:h-full w-full md:justify-center md:items-center py-10">
            <div className="flex w-full relative md:overflow-hidden ">
              <BackgroundImage />
            </div>
          </div>
        </div>
      </div>
      <div className="text-white px-10 text-5xl sm:text-6xl 2xl:text-9xl py-10">
        decades of framework best practices, brought to the modern data stack
      </div>
      <div className="flex flex-col  px-10 py-5 space-y-5 lg:space-y-0 lg:flex-row lg:space-x-5">
        {features.map((feature, index) => {
          return (
            <div key={index} className="flex flex-col md:flex-row flex-1">
              <div className="flex flex-col md:flex-1">
                <div className="text-action-primary text-2xl">
                  {feature.heading}
                </div>
                <div className="text-typography-primary my-3">
                  {feature.description}
                </div>
              </div>
              
            </div>
          )
        })}
      </div>
      <div>
        <div className="text-white px-10 text-5xl sm:text-6xl 2xl:text-9xl py-10">
          truly modern, truly open stack
        </div>
        <div className="h-full flex flex-col md:flex-row flex-grow md:justify-center md:items-center">
          <div className="flex flex-auto md:flex-1 flex-row md:h-full w-full md:justify-center md:items-center py-10">
            <div className="flex w-full relative md:overflow-hidden ">
              <Image  src={"/laptop.png"} className="" width={1024} height={1024} alt="developer in action" />
            </div>
          </div>
          <div className="text-white flex-col px-10 md:flex-1 space-y-5">
            {
              stack.map((item, index) => {
                return (
                  <div key={index} className="flex flex-col md:flex-row flex-1">
                    <div className="flex flex-col md:flex-1">
                      <div className="text-action-primary text-2xl">
                        {item.name}
                      </div>
                      <div className="text-typography-primary my-3">
                        {item.description}
                      </div>
                    </div>
                    
                  </div>
                )
              })
            }
            
          </div>
        </div>
      </div>

      <div className=" pt-32 w-screen flex grow-1 flex-col md:pt-16 ">
        <div className="h-full flex flex-col md:flex-row flex-grow md:justify-center md:items-center">
          <div className="text-white flex-col px-5 md:flex-1 ">
            <div className="px-5 text-5xl sm:text-6xl 2xl:text-9xl">
              start building today
            </div>
            <div className="flex flex-col grow md:flex-1 p-5 space-y-5">
              <div className="text-typography-primary my-3">
                Start building your data-intensive application today. Igloo is free to use and open source. If you&apos;d like to contribute, check out our github or join our discord.
              </div>
              <div>
                <CodeBlockCTA />
              </div>
            </div>
          </div>
          <div className="flex flex-auto md:flex-1 flex-row  w-full md:justify-center md:items-center py-10">
            <div className="flex w-full relative md:overflow-hidden ">
              <Image  src={"/hoodie.png"} className="" width={1024} height={1024} alt="developer in action" />
            </div>
          </div>
        </div>
      </div>


      <div className="flex sm:flex-row content-center grow flex-col gap-y-6 mt-6 px-10 py-10 sm:py-5">
        <RightsComponent />
        <LogoComponent />
      </div>
    </div>
  );
}
