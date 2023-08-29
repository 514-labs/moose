'use client'
import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";

export const RightsComponent = () => {
  const titleRef = React.useRef(null);

  useLayoutEffect(() => {
    let ctx = gsap.context(() => {

      const tl = gsap.timeline();

      gsap.set(titleRef.current, { perspective: 400 });  
      gsap.set(titleRef.current, { visibility: "visible" });

      tl.from(titleRef.current, {
        delay: 2,
        duration: 1,
        opacity: 0,
        scale: 0,
        y: 80,
        rotationX: 180,
        transformOrigin: "0% 50% 50",
        ease: "expo.out",
        });    
    });
    return () => {
      ctx.revert();
    }
  }, []);

  return (
    <div className="flex grow flex-wrap flex-row sm:justify-start  h-full content-center sm:-order-none order-3">
      <span className="text-white sm:text-start invisible" ref={titleRef}> ©2023 fiveonefour inc all rights reserved</span>
    </div>
  );
};
