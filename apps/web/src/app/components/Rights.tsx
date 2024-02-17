"use client";
import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";

export const RightsComponent = () => {
  const titleRef = React.useRef(null);

  useLayoutEffect(() => {
    const ctx = gsap.context(() => {
      const tl = gsap.timeline();

      gsap.set(titleRef.current, { perspective: 400 });
      gsap.set(titleRef.current, { visibility: "visible" });

      tl.from(titleRef.current, {
        y: "-50%",
        opacity: 0,
        stagger: { each: 0.02 },
      });
    });
    return () => {
      ctx.revert();
    };
  }, []);

  return (
    <div className="flex grow flex-wrap flex-row sm:justify-start  h-full content-center sm:-order-none order-3">
      <span className="text-white sm:text-start invisible" ref={titleRef}>
        {" "}
        ©2023 fiveonefour labs inc all rights reserved
      </span>
    </div>
  );
};
