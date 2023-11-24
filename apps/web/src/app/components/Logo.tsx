'use client'
import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import Image from 'next/image';

gsap.registerPlugin(SplitText);

export const LogoComponent = () => {
  const imgageRef = React.useRef(null);

  useLayoutEffect(() => {
    let ctx = gsap.context(() => {

      const tl = gsap.timeline();

      gsap.set(imgageRef.current, { perspective: 400 });
      gsap.set(imgageRef.current, { visibility: "visible" });

      tl.from(imgageRef.current, {
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
    <div  className="flex grow flex-row sm:justify-end sm:content-center sm:-order-none order-3">
      <div>
        <Image
          className="invisible"
          ref={imgageRef}
          src="/logo-514.svg"
          width={40}
          height={40}
          alt="Logo of the company" />
      </div>
    </div>
  );
};
