'use client'
import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";
import Image from "next/image";


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
    // <div className="bg-[url('/bg_igloo_image_person_02_4x.webp')] brightness-50 flex h-full" ref={imageRef} />
    <Image  src={'/hero.png'} className="invisible" width={1024} height={1024} alt="developer in action" ref={imageRef}/>
  );
};
