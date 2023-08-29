'use client'
import { Button } from "ui";
import { gsap } from "gsap";
import React, { useLayoutEffect } from "react";
import { RightsComponent } from "./RightsComponent";
import { LogoComponent } from "./LogoComponent";


const CodeBlockCTA = () => {
  return (
    <div className="flex flex-col lg:flex-row">
     <CodeBlock />
     <Button >copy</Button>
    </div>
  )
}

const CodeBlock = () => {
  return (
    <div className="flex flex-row items-center justify-center bg-base-black-250">
      <span className="font-mono py-3 px-6 text-typography-primary "> npx create-igloo-app</span>
    </div>
  )
}

export const CTASection = () => {
  const boxRef = React.useRef(null);
  const blurRef = React.useRef(null);

  useLayoutEffect(() => {
    let ctx = gsap.context(() => {

      const tl = gsap.timeline();

      gsap.set(boxRef.current, { perspective: 400 });  
      gsap.set(boxRef.current, { visibility: "visible" });
      gsap.set(blurRef.current, { visibility: "visible" });

      tl.from(boxRef.current, {
        opacity: 0,
        stagger: { each: 0.02 },
        });  

      tl.from(
        blurRef, {
          opacity: 0,
          stagger: { each: 0.02 },
          }
      )
    });
    return () => {
      ctx.revert();
    }
  }, []);

  return (
    <div className="p-5 relative overflow-clip invisible" ref={boxRef}>
      <div className="w-full absolute h-full z-0 left-0 top-0 bg-black" ref={blurRef} />
      <div className="relative z-10 flex flex-col md:flex-row">
        <div className="text-typography-secondary text-2xl my-3 flex grow flex-1">
          start building today
        </div>
        <div className="flex flex-col grow flex-1">
          <div className="text-typography-primary my-3">
            igloo is a framework to build data-intensive apps using typescript and sql. it comes with all the typescript primitives you'll need to build a fully featured app that's secure and scales. igloo also provides comes with a CLI to help you be productive while you build your data-intensive application right along side your web app on local machine. no need to configure clusters and networking to start building.
          </div>
          <div>
            <CodeBlockCTA />
          </div>
          
        </div>
        
      </div>
      <div className="flex sm:flex-row content-center grow flex-col gap-y-6">
            <RightsComponent />
            <LogoComponent />
          </div>
    </div>
  );
};
