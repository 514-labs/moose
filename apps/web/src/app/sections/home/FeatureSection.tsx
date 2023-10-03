import React from "react";

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

const featureSection = {
  heading: "decades of software best practices, brought to the modern data stack",
  features: features
}

export const FeatureSection = () => {
  return (
    <div className="my-24">
    <div className="text-white px-10 text-5xl sm:text-6xl 2xl:text-9xl my-24">
      {featureSection.heading}
    </div>
    <div className="flex flex-col  px-10 py-5 space-y-5 lg:space-y-0 lg:flex-row lg:space-x-5">
      {featureSection.features.map((feature, index) => {
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
        );
      })}
    </div>
  </div>)
};
