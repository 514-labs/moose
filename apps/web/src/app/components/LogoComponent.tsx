'use client'
import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import { AnimatedImage } from "./AnimatedImage";
import { AnimatedComponent } from "./AnimatedComponent";
import Image from 'next/image';

gsap.registerPlugin(SplitText);

export const LogoComponent = () => {
  const imgageRef = React.useRef(null);

  // useLayoutEffect(() => {
  //   let ctx = gsap.context(() => {

  //     const tl = gsap.timeline();

  //     gsap.set(imgageRef.current, { perspective: 400 });
  //     gsap.set(imgageRef.current, { visibility: "visible" });

  //     tl.from(imgageRef.current, {
  //       delay: 2,
  //       duration: 1,
  //       opacity: 0,
  //       scale: 0,
  //       y: 80,
  //       rotationX: 180,
  //       transformOrigin: "0% 50% 50",
  //       ease: "expo.out",
  //       });    
  //   });
  //   return () => {
  //     ctx.revert();
  //   }
  // }, []);

  return (
    <div  className="flex grow flex-row sm:justify-end sm:content-center sm:-order-none order-3">
      <div>
      <AnimatedComponent position={1.5} >
          <Image 
            // className="invisible"
            ref={imgageRef}
            src="/logo-moose-black.svg"
            width={36}
            height={36}
            alt="Logo of the product" />      
         </AnimatedComponent>
      </div>
    </div>
  );
};
