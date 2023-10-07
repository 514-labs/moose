'use client'
import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";
import Image, { StaticImageData } from "next/image";

interface AnimateImageProps {
  src: string | StaticImageData,
  alt: string,
  priority?: boolean,
  onScroll?: boolean
}

export const AnimateImage = ({src, alt, priority}: AnimateImageProps) => {
  const imageRef = React.useRef(null);

  useLayoutEffect(() => {
    let ctx = gsap.context(() => {
      
      gsap.from(imageRef.current,{
        scrollTrigger: {
          trigger: imageRef.current,
          onEnter: () => {
            gsap.set(imageRef.current, { visibility: "visible" });
          }
        },
        opacity: 0,
        y:100,
        duration: 1,
        ease: "quint",
        delay: 1.0,

      });
    });
    return () => {
      ctx.revert();
    };
  }, []);

  return (
    <Image  src={src} className="invisible object-cover" fill sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw" alt={alt} ref={imageRef} priority={priority}/>
  );
};


