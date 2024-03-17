"use client";
import React, { useLayoutEffect } from "react";
import { AnimateImage } from "../../components/AnimateImage";
import { CodeBlockCTA } from "../../components/CodeBlockCTA";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import heroImg from "../../../../public/03_4x.webp";

export const heroContent = {
  heading: "the application framework for the modern data stack",
  description:
    "Moose is a batteries-included framework for building data-intensive applications using Typescript or Python, and SQL. It comes with a powerful CLI to help automate development tasks, an intuitive abstraction to help you build quickly, and a streamlined local development workflow.",
};

interface HeroSectionProps {
  identifier: string;
}

export const HeroSection = ({ identifier }: HeroSectionProps) => {
  const headingRef = React.useRef(null);
  const descriptionRef = React.useRef(null);

  useLayoutEffect(() => {
    const ctx = gsap.context(() => {
      const tl = gsap.timeline();
      const splitText = new SplitText(headingRef.current, {
        type: "words, chars",
      });
      const splitTextChars = splitText.chars;

      const splitTextByLines = new SplitText(descriptionRef.current, {
        type: "lines",
      });
      const splitTextLines = splitTextByLines.lines;

      gsap.set(headingRef.current, { visibility: "visible" });
      gsap.set(descriptionRef.current, { visibility: "visible" });

      tl.from(
        splitTextChars,
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
    <div className="grid pt-24 mb-24 grid-cols-1 grid-row-2 space-y-24 md:space-y-0 md:grid-cols-2 md:grid-row-1  place-items-center">
      <div className="flex text-white flex-col px-5 md:flex-1 md:justify-center">
        <div className="px-5 text-5xl sm:text-6xl 2xl:text-8xl 3xl:text-9xl">
          <span ref={headingRef} className="invisible">
            {heroContent.heading}
          </span>
        </div>
        <div className="flex flex-col grow md:flex-1 p-5 space-y-5">
          <div className="text-typography-primary my-3">
            <span className="invisible" ref={descriptionRef}>
              {heroContent.description}
            </span>
          </div>
          <div>
            <CodeBlockCTA identifier={identifier} />
          </div>
        </div>
      </div>
      <div className="h-full w-full min-h-[30vh] relative lg:min-h-[50vh] xl:min-h-[75vh]">
        <AnimateImage src={heroImg} alt="developer lifestyle" priority />
      </div>
    </div>
  );
};
