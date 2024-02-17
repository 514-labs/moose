"use client";
import React, { useLayoutEffect } from "react";
import { AnimateImage } from "../../components/AnimateImage";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import sectionImg from "../../../../public/04_4x.webp";

gsap.registerPlugin(SplitText);
gsap.registerPlugin(ScrollTrigger);

const stack = [
  {
    name: "Fully integrated data stack",
    description:
      "We've composed moose from best-in-class data infrastructure to enable you to run your entire modern data stack on your local machine. No more configuring connections to start building.",
  },
  {
    name: "Best-in-class streaming",
    description:
      "We've created a highly performant and scalable data capture stack that scales with your data volumes and is lightweight enough to run locally. Rust ingestion points & native support for Redpanda.",
  },
  {
    name: "Modern analyics storage",
    description:
      "We use the latest generation of analytics storage to guarantee performance and a great local development experience. Native clickhouse support with DuckDB and Delta Lake coming soon.",
  },
  {
    name: "Intuitive data modeling",
    description:
      "We love working with Prisma in when building web apps. We've brought their modeling language to the modern data stack to help you create intuitive and readable data models.",
  },
];

const howItWorksSection = {
  heading: "truly modern, truly open stack",
  stack: stack,
};

export const HowItWorksSection = () => {
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
        0,
      );

      tl.from(
        splitTextLines,
        {
          y: "-10",
          opacity: 0,
          ease: "quint",
          stagger: { each: 0.03 },
        },
        1,
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
    <div>
      <div className="text-white px-10 text-5xl my-24 sm:text-6xl 2xl:text-8xl 3xl:text-9xl">
        <span className="invisible" ref={headingRef}>
          {howItWorksSection.heading}
        </span>
      </div>
      <div className="grid mb-24 grid-cols-1 grid-row-2 space-y-24 md:space-y-0 md:grid-cols-2 md:grid-row-1  place-items-center">
        <div className="h-full w-full min-h-[30vh] relative ">
          <AnimateImage src={sectionImg} alt="developer lifestyle" priority />
        </div>
        <div className="text-white flex-col px-10 md:flex-1 space-y-5">
          {howItWorksSection.stack.map((item, index) => {
            return (
              <div key={index} className="flex flex-col md:flex-row flex-1">
                <div className="flex flex-col md:flex-1">
                  <div className="text-action-primary text-2xl">
                    <span
                      className="invisible"
                      ref={(el) => (featureHeadingRef.current[index] = el)}
                    >
                      {item.name}
                    </span>
                  </div>
                  <div className="text-typography-primary my-3">
                    <span
                      className="invisible"
                      ref={(el) => (featureDescriptionRef.current[index] = el)}
                    >
                      {item.description}
                    </span>
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
