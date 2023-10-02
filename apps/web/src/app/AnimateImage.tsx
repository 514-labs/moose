'use client'
import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";
import Image from "next/image";

interface AnimateImageProps {
  src: string,
  width: number,
  height: number,
  alt: string,
  priority?: boolean,
}

export const AnimateImage = ({src, width, height, alt, priority}: AnimateImageProps) => {
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
    <Image  src={src} className="invisible" width={width} height={height} alt={alt} ref={imageRef} priority={priority}/>
  );
};


