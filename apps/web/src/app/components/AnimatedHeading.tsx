"use client";

import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);
gsap.registerPlugin(SplitText);

interface HeadingProps {
  content: string;
  size: Size;
  onScroll?: boolean; // Triggers when the user scrolls to the element
  triggerRef?: React.MutableRefObject<HTMLDivElement>; // The ref to the element that triggers the animation. Defaults to the element itself
  className?: string;
  position?: number;
}

type Size = "display" | "display-md" | "express" | "heading";

const getDefaultStyle = (size: Size) => {
  if (size === "display") {
    return "text-typography text-5xl sm:text-7xl md:text-8xl lg:text-9xl 4xl:text-10xl 4xl:leading-none text-black";
  }

  if (size === "display-md") {
    return "text-typography text-5xl 2xl:text-6xl 3xl:text-7xl text-black";
  }

  if (size === "express") {
    return "text-typography text-center text-3xl lg:text-4xl 2xl:text-6xl 3xl:text-7xl text-black";
  }

  if (size === "heading") {
    return "text-3xl text-action";
  }
};

const getStyle = (className: string, size: Size) => {
  if (className) {
    return className + " " + getDefaultStyle(size);
  } else {
    return getDefaultStyle(size);
  }
};

export const AnimatedHeading = ({
  content,
  size,
  onScroll,
  triggerRef,
  className,
  position,
}: HeadingProps) => {
  const headingRef = React.useRef(null);
  const computedTriggerRef = triggerRef || headingRef;
  let computedPosition = position || 0;

  useLayoutEffect(() => {
    const ctx = gsap.context(() => {
      const tl = onScroll
        ? gsap.timeline({
            scrollTrigger: {
              trigger: computedTriggerRef.current,
              onEnter: (self) => {
                gsap.set(headingRef.current, { visibility: "visible" });
                if (self.getVelocity() > 0) {
                  computedPosition = 0;
                }
              },
            },
          })
        : gsap.timeline();

      if (!onScroll) {
        tl.set(headingRef.current, { visibility: "visible" });
      }

      const splitText = new SplitText(headingRef.current, {
        type: "words, chars, lines",
      });
      const splitTextChars = splitText.chars;

      const stagger = size === "express" ? 0.01 : 0.02;

      const animation = {
        y: "10",
        opacity: 0,
        duration: 1,
        ease: "quint",
        stagger: { each: stagger },
      };

      tl.from(splitTextChars, animation, computedPosition);
    });
    return () => {
      ctx.revert();
    };
  }, []);

  return (
    <div className={getStyle(className, size)}>
      <span className="invisible" ref={headingRef}>
        {content}
      </span>
    </div>
  );
};
