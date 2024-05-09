"use client";

import { cn } from "@ui/lib/utils";
import { Copy } from "lucide-react";
import { ReactNode } from "react";
import { Text } from "./standard";

import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import { useLayoutEffect } from "react";
import { useRef } from "react";
import { MutableRefObject } from "react";

const copyPasteAnimation = (
  inboundRef: MutableRefObject<any>,
  outboundRef: MutableRefObject<any>,
  wrapperRef: MutableRefObject<any>,
  onCopy: (() => void) | undefined,
) => {
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
      if (outboundRef.current.innerText.trim().length > 0) {
        onCopy && onCopy();
        navigator.clipboard.writeText(outboundRef.current.innerText.trim());
      }

      gsap.set(outboundRef.current, { visibility: "hidden" });
      gsap.set(inboundRef.current, { visibility: "visible" });

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
        gsap.set(inboundRef.current, { visibility: "hidden" });
      });

      gsap.delayedCall(1, () => {
        gsap.set(outboundRef.current, { visibility: "visible" });
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
  });

  return () => {
    ctx.revert();
  };
};

export const CodeSnippet = ({
  children,
  className,
  onCopy,
}: {
  children: ReactNode;
  className?: string;
  onCopy?: () => void;
}) => {
  const inboundRef = useRef(null);
  const outboundRef = useRef(null);
  const wrapperRef = useRef(null);

  useLayoutEffect(() => {
    copyPasteAnimation(inboundRef, outboundRef, wrapperRef, onCopy);
  }, []);

  return (
    <div
      ref={wrapperRef}
      className={cn(
        "text-primary bg-muted rounded-md py-5 px-6 flex flex-row gap-5 cursor-pointer items-center justify-center",
        className,
      )}
    >
      <Text className="grow my-0 relative">
        <span ref={outboundRef}>{children}</span>
        <span ref={inboundRef} className="absolute top-0 invisible">
          copied to clipboard
        </span>
      </Text>
      <div>
        <Copy strokeWidth={2.5} />
      </div>
    </div>
  );
};
