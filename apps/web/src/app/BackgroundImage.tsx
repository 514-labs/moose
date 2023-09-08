'use client'
import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";


export const BackgroundImage = () => {
  const imageRef = React.useRef(null);

  useLayoutEffect(() => {
    let ctx = gsap.context(() => {
      gsap.set(imageRef.current, { visibility: "visible" });
      
      gsap.from(imageRef.current,{
        opacity: 0,
        duration: 2,
      });
    });
    return () => {
      ctx.revert();
    };
  }, []);

  return (
    <div className="invisible sm:w-full sm:h-full sm:fixed bg-[url('/bg_igloo_image_person_02_4x.webp')] bg-bottom bg-cover brightness-50" ref={imageRef} />
  );
};
