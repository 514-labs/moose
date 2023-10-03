import React from "react";
import { AnimateImage } from "../../components/AnimateImage";


const stack = [
  {
    "name": "Fully integrated data stack",
    "description": "We've composed igloo from best-in-class data infrastructure to enable you to run your entire modern data stack on your local machine. No more configuring connections to start building.",
  },
  {
    "name": "Best-in-class streaming",
    "description": "We've created a highly performant and scalable data capture stack that scales with your data volumes and is lightweight enough to run locally. Rust ingestion points & native support for Redpanda.",
  },
  {
    "name": "Modern analyics storage",
    "description": "We use the latest generation of analytics storage to guarantee performance and a great local development experience. Native clickhouse support with DuckDB and Delta Lake coming soon.",
  },
  {
    "name": "Intuitive data modeling",
    "description": "We love working with Prisma in when building web apps. We've brought their modeling language to the modern data stack to help you create intuitive and readable data models.",
  },
]

const howItWorksSection = {
  heading: "truly modern, truly open stack",
  stack: stack
}




export const HowItWorksSection = () => {
  return (
    <div>
      <div className="text-white px-10 text-5xl sm:text-6xl 2xl:text-9xl my-24">
        {howItWorksSection.heading}
      </div>
      <div className="h-full flex flex-col md:flex-row flex-grow md:justify-center md:items-center">
        <div className="flex flex-auto md:flex-1 flex-row md:h-full w-full md:justify-center md:items-center">
          <div className="flex w-full relative md:overflow-hidden ">
            <AnimateImage src="/laptop.png" width={1024} height={1024} alt="developer in action" />
          </div>
        </div>
        <div className="text-white flex-col px-10 md:flex-1 space-y-5 my-24">
          {howItWorksSection.stack.map((item, index) => {
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
            );
          })}
        </div>
      </div>
    </div>
  );
};
