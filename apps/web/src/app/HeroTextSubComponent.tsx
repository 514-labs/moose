"use client";
import { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import React from "react";

gsap.registerPlugin(SplitText);

export const HeroTextSubComponent = () => {
  const titleRef = React.useRef(null);

  useLayoutEffect(() => {
    const ctx = gsap.context(() => {
      const tl = gsap.timeline();
      const splitText = new SplitText(titleRef.current, {
        type: "words, chars",
      });
      const splitTextChars = splitText.chars;

      gsap.set(titleRef.current, { perspective: 400 });
      gsap.set(titleRef.current, { visibility: "visible" });

      tl.from(splitTextChars, {
        delay: 1,
        duration: 1,
        opacity: 0,
        scale: 0,
        y: 80,
        rotationX: 180,
        transformOrigin: "0% 50% -50",
        ease: "expo.out",
        stagger: 0.01,
      });
    });
    return () => {
      ctx.revert();
    };
  }, []);

  return (
    <div className="w-full 2xl:text-9xl sm:text-5xl text-4xl md:text-right h-full">
      <span className="inline-block invisible" ref={titleRef}>
        creating the future of{" "}
        <span className="whitespace-nowrap">data-intensive</span> app
        development
      </span>
    </div>
  );
};
