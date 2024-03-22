"use client";
import { Button } from "ui";
import { gsap } from "gsap";
import React, { useLayoutEffect } from "react";
import { SplitText } from "gsap/SplitText";
import { sendClientEvent } from "../events/sendClientEvent";

interface CodeBlockCTAProps {
  identifier: string;
}

export const CodeBlockCTA = ({ identifier }: CodeBlockCTAProps) => {
  const inboundRef = React.useRef(null);
  const outboundRef = React.useRef(null);
  const wrapperRef = React.useRef(null);

  useLayoutEffect(() => {
    const ctx = gsap.context(() => {
      const tl = gsap.timeline();
      const splitTextOutbound = new SplitText(outboundRef.current, {
        type: "words, chars, lines",
      });
      const splitTextCharsOutbount = splitTextOutbound.chars;

      const splitTextInbound = new SplitText(inboundRef.current, {
        type: "words, chars, lines",
      });
      const splitTextCharsInbound = splitTextInbound.chars;

      wrapperRef.current.addEventListener("click", () => {
        // copy to clipboard
        navigator.clipboard.writeText("npx create-moose-app");

        gsap.set(outboundRef.current, { display: "none" });
        gsap.set(inboundRef.current, { display: "block" });

        gsap.fromTo(
          splitTextCharsInbound,
          {
            opacity: 0,
            stagger: { each: 0.01 },
          },
          {
            opacity: 1,
            stagger: { each: 0.03 },
          },
        );

        gsap.delayedCall(1, () => {
          gsap.set(inboundRef.current, { display: "none" });
        });

        gsap.delayedCall(1, () => {
          gsap.set(outboundRef.current, { display: "block" });
          gsap.fromTo(
            splitTextCharsOutbount,
            {
              opacity: 0,
              stagger: { each: 0.01 },
            },
            {
              opacity: 1,
              stagger: { each: 0.03 },
            },
          );
        });
      });

      gsap.set(wrapperRef.current, { visibility: "visible" });

      tl.from(wrapperRef.current, {
        opacity: 0,
        y: 30,
        duration: 1,
        delay: 1,
        stagger: 2,
      });
    });

    return () => {
      ctx.revert();
    };
  }, []);

  return (
    <div
      className="flex flex-col lg:flex-row cursor-pointer invisible gap-3"
      ref={wrapperRef}
    >
      <div className="flex flex-row items-center justify-center bg-black/10 h-13 rounded-xl ">
        <span
          className="px-8 py-4 sm:py-6 w-72 text-center text-typography text-black/100"
          ref={outboundRef}
        >
          {" "}
          npx create-moose-app
        </span>
        <span
          className="px-8 py-4 sm:py-6 w-72 text-center text-typography hidden"
          ref={inboundRef}
        >
          {" "}
          copied to clipboard
        </span>
      </div>
      <Button
        className="py-4 text-center font-medium no-underline bg-action-primary bg-action-white bg-black/100 text-gray-300 hover:bg-gray-900 sm:inline-block sm:grow-0 md:py-6 md:px-10 md:text-lg md:leading-8 rounded-xl"
        onClick={() => sendClientEvent("cta-click-create-app", identifier, {})}
      >
        copy
      </Button>
    </div>
  );
};
