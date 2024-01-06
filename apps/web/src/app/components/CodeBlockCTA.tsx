'use client';
import { Button } from "ui";
import { gsap } from "gsap";
import React, { useLayoutEffect } from "react";
import { SplitText } from "gsap/SplitText";
import { AnimatedComponent } from "../components/AnimatedComponent";

export const CodeBlockCTA = () => {
  const inboundRef = React.useRef(null);
  const outboundRef = React.useRef(null);
  const wrapperRef = React.useRef(null);

  useLayoutEffect(() => {
    let ctx = gsap.context(() => {

      const tl = gsap.timeline();
      const splitTextOutbound = new SplitText(outboundRef.current, { type: "words, chars" });
      const splitTextCharsOutbount = splitTextOutbound.chars;

      const splitTextInbound = new SplitText(inboundRef.current, { type: "words, chars" });
      const splitTextCharsInbound = splitTextInbound.chars;

      wrapperRef.current.addEventListener("click", () => {
        // copy to clipboard
        navigator.clipboard.writeText("npx create-moose-app");

        gsap.set(outboundRef.current, { display: "none" });
        gsap.set(inboundRef.current, { display: "block" });

        gsap.fromTo(splitTextCharsInbound, {
          opacity: 0,
          stagger: { each: 0.02 },
        }, {
          opacity: 1,
          stagger: { each: 0.02 },
        });

        gsap.delayedCall(1, () => {
          gsap.set(inboundRef.current, { display: "none" });
        });

        gsap.delayedCall(1, () => {
          gsap.set(outboundRef.current, { display: "block" });
          gsap.fromTo(splitTextCharsOutbount, {
            opacity: 0,
            stagger: { each: 0.02 },
          }, {
            opacity: 1,
            stagger: { each: 0.02 },
          });
        }

        );
      });

      gsap.set(wrapperRef.current, { visibility: "visible" });


      tl.from(wrapperRef.current, {
        opacity:0,
        y: 30,
        duration: 1,
        delay: 1.4,
        stagger: 0.04
      })
    });


    return () => {
      ctx.revert();
    };
  }, []);

  return (
    <div className="flex flex-col lg:flex-row cursor-pointer invisible" ref={wrapperRef}>
      <div className="flex flex-row items-center justify-center sm:justify-start bg-white/10 w-full h-13 ">
        <span className="font-mono py-4 px-6 text-typography-secondary text-gray-300 " ref={outboundRef}> npx create-moose-app</span>
        <span className="font-mono py-4 px-6 text-typography-primary hidden" ref={inboundRef}> copied to clipboard</span>
      </div>
        <Button className="py-4 text-center font-medium no-underline bg-action-primary bg-action-white text-black bg-gray-300 hover:bg-gray-100 sm:inline-block sm:grow-0 md:py-6 md:px-10 md:text-lg md:leading-8">copy</Button>
    </div>
  );
};
