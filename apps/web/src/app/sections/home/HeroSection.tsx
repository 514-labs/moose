import React from "react";
import { AnimateImage } from "../../components/AnimateImage";
import { CodeBlockCTA } from "../../components/CodeBlockCTA";

export const heroContent = {
  heading: "the application framework for the modern data stack",
  description: "Igloo is a batteries-included framework for building data-intensive applications using Typescript or Python, and SQL. It comes with a powerful CLI to help automate development tasks, an intuitive abstraction to help you build quickly, and a streamlined local development workflow."
}

export const HeroSection = () => {
  return <div className=" w-screen flex grow-1 flex-col pt-24 mb-24">
    <div className="h-full flex flex-col md:flex-row flex-grow md:justify-center md:items-center">
      <div className="text-white flex-col px-5 md:flex-1 ">
        <div className="px-5 text-5xl sm:text-6xl 2xl:text-9xl">
          {heroContent.heading}
        </div>
        <div className="flex flex-col grow md:flex-1 p-5 space-y-5">
          <div className="text-typography-primary my-3">
            {heroContent.description}
          </div>
          <div>
            <CodeBlockCTA />
          </div>
        </div>
      </div>
      <div className="flex flex-auto md:flex-1 flex-row md:h-full w-full md:justify-center md:items-center">
        <div className="flex w-full relative md:overflow-hidden ">
          <AnimateImage src="/hero.png" width={1024} height={1024} alt="developer lifestyle" priority />
        </div>
      </div>
    </div>
  </div>;
};
