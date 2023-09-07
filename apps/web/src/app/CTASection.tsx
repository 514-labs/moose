'use client'
import { Button } from "ui";
import { gsap } from "gsap";
import React, { useLayoutEffect } from "react";
import { RightsComponent } from "./RightsComponent";
import { LogoComponent } from "./LogoComponent";
import { SplitText } from "gsap/SplitText";



const copyAnimation = () => {
  
}



const CodeBlockCTA = () => {
  const inboundRef = React.useRef(null);
  const outboundRef = React.useRef(null);
  const wrapperRef = React.useRef(null);

  useLayoutEffect(() => {
    let ctx = gsap.context(() => {

      // const tl = gsap.timeline();
      const splitTextOutbound = new SplitText(outboundRef.current, { type: "words, chars" });
      const splitTextCharsOutbount = splitTextOutbound.chars;

      const splitTextInbound = new SplitText(inboundRef.current, { type: "words, chars" });
      const splitTextCharsInbound = splitTextInbound.chars;

      wrapperRef.current.addEventListener("click", () => {
        // copy to clipboard
        navigator.clipboard.writeText("npx create-igloo-app");

        gsap.set(outboundRef.current, {display: "none"})
        gsap.set(inboundRef.current, {delay:  0.2, display: "block"})

        gsap.fromTo(splitTextCharsInbound, {

          opacity: 0,
          stagger: { each: 0.02 },
        }, {

          opacity: 1,
          stagger: { each: 0.02 },
        });

        gsap.delayedCall(1, () => {
          gsap.set(inboundRef.current, { display: "none"})
        })

        gsap.delayedCall(1, () => {
          gsap.set(outboundRef.current, {display: "block"})
          gsap.fromTo(splitTextCharsOutbount, {

            opacity: 0,
            stagger: { each: 0.02 },
          }, {

            opacity: 1,
            stagger: { each: 0.02 },
          }); 
        }

        )
      })
    });
    return () => {
      ctx.revert();
    }
  }, []); 

  return (
    <div className="flex flex-col lg:flex-row cursor-pointer" ref={wrapperRef}>
    <div className="flex flex-row items-center justify-center bg-base-black-250 md:w-72" >
      <span className="font-mono py-3 px-6 text-typography-primary " ref={outboundRef}> npx create-igloo-app</span>
      <span className="font-mono py-3 px-6 text-typography-secondary hidden" ref={inboundRef}> copied to clipboard</span>
    </div>
     <Button>copy</Button>
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
    <div className="sm:p-8 p-4 relative overflow-clip invisible" ref={boxRef}>
      <div className="w-full absolute h-full z-0 left-0 top-0 bg-black" ref={blurRef} />
      <div className="relative z-10 flex flex-col md:flex-row">
        <div className="text-typography-secondary text-2xl my-3 flex grow flex-1">
          start building today
        </div>
        <div className="flex flex-col grow flex-1">
          <div className="text-typography-primary my-3">
            Igloo is a framework to build data-intensive apps using typescript and sql. It comes with all the typescript primitives you&apos;ll need to build a fully featured app that&apos;s secure and scales. Igloo also provides comes with a CLI to help you be productive while you build your data-intensive application right along side your web app on local machine. No need to configure clusters and networking to start building.
          </div>
          <div>
            <CodeBlockCTA />
          </div>
          
        </div>
        
      </div>
      <div className="flex sm:flex-row content-center grow flex-col gap-y-6 mt-6">
        <RightsComponent />
        <LogoComponent />
      </div>
    </div>
  );
};
