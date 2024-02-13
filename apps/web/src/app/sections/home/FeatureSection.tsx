"use client";
import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(SplitText);
gsap.registerPlugin(ScrollTrigger);

const features = [
  {
    heading: "Local first",
    description:
      "Build your application locally, and deploy to the cloud when you're ready. No need to configure a cloud environment to get started.",
  },
  {
    heading: "Languages you love",
    description:
      "Write your application in Typescript or Python, build your data models in Prisma and use SQL to interact with your OLAP database.",
  },
  // {
  //   heading: "Realtime ready",
  //   description: "Build realtime data-intensive applications with ease. Moose comes with realtime support out of the box. ",
  // },
  {
    heading: "Easy to test",
    description:
      "Write tests for your application using the languages you love. Moose supports your favorite frameworks in Typescript and Python.",
  },
  {
    heading: "Zero configuration",
    description:
      "Get up and running with your application in minutes. Moose comes with a powerful CLI to help you automate development tasks.",
  },
  {
    heading: "Abstracted complexity",
    description:
      "Moose abstractions help you focus on building your end-to-end application without worrying about the underlying infrastructure.",
  },
];

const featureSection = {
  heading:
    "decades of software best practices, brought to the modern data stack",
  features: features,
};

export const FeatureSection = () => {
  const headingRef = React.useRef(null);

  const featureHeadingRef = React.useRef([]);
  const featureDescriptionRef = React.useRef([]);

  useLayoutEffect(() => {
    const ctx = gsap.context(() => {
      const tl = gsap.timeline({
        scrollTrigger: {
          trigger: featureHeadingRef.current,
          onEnter: () => {
            gsap.set(featureHeadingRef.current, { visibility: "visible" });
            gsap.set(featureDescriptionRef.current, { visibility: "visible" });
          },
        },
      });
      const splitTextHeading = new SplitText(headingRef.current, {
        type: "words, chars",
      });
      const splitTextHeadingChars = splitTextHeading.chars;

      const splitTextFeatureHeading = new SplitText(featureHeadingRef.current, {
        type: "words, chars",
      });
      const splitTextFeatureHeadingChars = splitTextFeatureHeading.chars;

      const splitTextByLines = new SplitText(featureDescriptionRef.current, {
        type: "lines",
      });
      const splitTextLines = splitTextByLines.lines;

      gsap.from(splitTextHeadingChars, {
        scrollTrigger: {
          trigger: headingRef.current,
          onEnter: () => {
            gsap.set(headingRef.current, { visibility: "visible" });
          },
        },
        y: "-20",
        opacity: 0,
        ease: "quint",
        stagger: { each: 0.03 },
      });

      tl.from(
        splitTextFeatureHeadingChars,
        {
          y: "-20",
          opacity: 0,
          ease: "quint",
          stagger: { each: 0.03 },
        },
        0
      );

      tl.from(
        splitTextLines,
        {
          y: "-10",
          opacity: 0,
          ease: "quint",
          stagger: { each: 0.03 },
        },
        1
      );

      tl.then(() => {
        splitTextByLines.revert();
      });
    });
    return () => {
      ctx.revert();
    };
  }, []);

  return (
    <div className="my-24">
      <div className="text-white px-10  my-24 text-5xl sm:text-6xl 2xl:text-8xl 3xl:text-9xl">
        <span className="invisible" ref={headingRef}>
          {featureSection.heading}
        </span>
      </div>
      <div className="flex flex-col  px-10 space-y-5 lg:space-y-0 lg:flex-row lg:space-x-5">
        {featureSection.features.map((feature, index) => {
          return (
            <div key={index} className="flex flex-col md:flex-row flex-1">
              <div className="flex flex-col md:flex-1">
                <div className="text-action-primary text-2xl">
                  <span
                    className="invisible"
                    ref={(el) => (featureHeadingRef.current[index] = el)}
                  >
                    {feature.heading}
                  </span>
                </div>
                <div className="text-typography-primary my-3">
                  <span
                    className="invisible"
                    ref={(el) => (featureDescriptionRef.current[index] = el)}
                  >
                    {feature.description}
                  </span>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
