'use client'
import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { Button } from "ui";

export const CTAComponent = () => {
  const buttonRef = React.useRef(null);

  useLayoutEffect(() => {
    let ctx = gsap.context(() => {

      const tl = gsap.timeline();

      gsap.set(buttonRef.current, { perspective: 400 }); 
      gsap.set(buttonRef.current, { visibility: "visible" });

      tl.from(buttonRef.current, {
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
    <div  className="flex grow flex-row sm:justify-end justify-center sm:-order-none order-1">
      <div ref={buttonRef} className="invisible flex grow sm:block sm:grow-0">
        <Button>get in touch</Button>
      </div>
    </div>
  );
};
