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

      // For some reason this section bounces if we revert the split text-line
      // tl.then(() => {
      //   splitTextByLines.revert()
      // })

    });
    return () => {
      ctx.revert();
    }
  }, []);

  return (
    <div>
    <div className="grid grid-cols-1 grid-row-2 md:grid-cols-2 md:grid-row-1 place-items-center">
      <div className="flex text-white my-24 flex-col px-5 order-last md:order-first md:flex-1 md:justify-center ">
        <div className="px-5 text-5xl sm:text-6xl 2xl:text-8xl 3xl:text-9xl">
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
      <div className="h-full w-full min-h-[30vh] relative lg:min-h-[50vh]">
          <AnimateImage src="/hoodie.png" alt="developer lifestyle" priority />
      </div>
    </div> 
    </div>
    
  );
};