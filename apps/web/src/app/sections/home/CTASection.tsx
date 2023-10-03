'use client'
import React, { useLayoutEffect } from "react";
import { AnimateImage } from "../../components/AnimateImage";
import { CodeBlockCTA } from "../../components/CodeBlockCTA";

import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(SplitText);
gsap.registerPlugin(ScrollTrigger);


export const ctaSection = {
  heading: "start building today",
  description: "Start building your data-intensive application today. Igloo is free to use and open source. If you'd like to contribute, check out our github or join our discord."

}

export const CTASection = () => {
  const headingRef = React.useRef(null);
  
  const featureDescriptionRef = React.useRef(null);

  useLayoutEffect(() => {
    let ctx = gsap.context(() => {

      const tl = gsap.timeline({
        scrollTrigger: {
          trigger: featureDescriptionRef.current,
          onEnter: () => {
            gsap.set(featureDescriptionRef.current, { visibility: "visible" });
          }
        },
      });

      const splitTextHeading = new SplitText(headingRef.current, { type: "words, chars" });
      const splitTextHeadingChars = splitTextHeading.chars;


      const splitTextByLines = new SplitText(featureDescriptionRef.current, {type: "lines"});
      const splitTextLines = splitTextByLines.lines;

      gsap.from(splitTextHeadingChars,{
        scrollTrigger: {
          trigger: headingRef.current,
          onEnter: () => {
            gsap.set(headingRef.current, { visibility: "visible" });
          }
        },
        y: "-20",
        opacity: 0,
        ease: "quint",
        stagger: { each: 0.03 },
      });

      tl.from(
        splitTextLines,
        {
          y: "-10",
          opacity: 0,
          ease: "quint",
          stagger: { each: 0.03 },
        },
        1
      )

    });
    return () => {
      ctx.revert();
    }
  }, []);

  return (
    <div className="w-screen flex grow-1 flex-col">
      <div className="h-full flex flex-col md:flex-row flex-grow md:justify-center md:items-center">
        <div className="text-white flex-col px-5 md:flex-1 ">
          <div className="px-5 text-5xl sm:text-6xl 2xl:text-9xl">
            <span className="invisible" ref={headingRef}>
              {ctaSection.heading}
            </span>
          </div>
          <div className="flex flex-col grow md:flex-1 p-5 space-y-5">
            <div className="text-typography-primary my-3">
              <span className="invisible" ref={featureDescriptionRef}>
                {ctaSection.description}
              </span>
            </div>
            <div>
              <CodeBlockCTA />
            </div>
          </div>
        </div>
        <div className="flex flex-auto md:flex-1 flex-row  w-full md:justify-center md:items-center mt-24">
          <div className="flex w-full relative md:overflow-hidden ">
            <AnimateImage src="/hoodie.png" width={1024} height={1024} alt="developer in style" />
          </div>
        </div>
      </div>
    </div>
  );
};
