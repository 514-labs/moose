'use client'
import { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import React from "react";

gsap.registerPlugin(SplitText);

export const HeroTextSubComponent = () => {
  const titleRef = React.useRef(null);

  useLayoutEffect(() => {
    let ctx = gsap.context(() => {

      const tl = gsap.timeline();
      const splitText = new SplitText(titleRef.current, { type: "words, chars" });
      const splitTextChars = splitText.chars;

      gsap.set(titleRef.current, { perspective: 400 });  
      gsap.set(titleRef.current, { visibility: "visible" });

      tl.from(splitTextChars,{
        y: "-50%",
        opacity: 0,
        stagger: { each: 0.02 },
        });   
    });
    return () => {
      ctx.revert();
    }
  }, []);

  return (
    <div className="w-full 2xl:text-8xl sm:text-5xl text-4xl md:text-right flex flex-col shrink">
      <span className="inline-block invisible" ref={titleRef}> a framework to build data-intensive apps</span>
    </div>
  );
};
