'use client'

import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";

gsap.registerPlugin(SplitText);

/**
 * Animates the children components as one animation block
 * 
 * Use this component to animate a group of components as one animation block. This could be a button or code block.
 * 
 * If you're trying to animate headings or paragraphs, use the AnimatedHeading or AnimatedDescription components instead.
 */



interface AnimationProps {
    children: React.ReactNode,
    onScroll?: boolean, // Triggers when the user scrolls to the element
    triggerRef?: React.MutableRefObject<HTMLDivElement>, // The ref to the element that triggers the animation. Defaults to the element itself
    className?: string,
    position?: number,
}



const getStyle = (className: string) => {
    if (className) {
        return className + "text-typography-primary my-3"
    } else {
        return "text-typography-primary my-3";
    }
}


export const AnimatedComponent = ({children, onScroll, triggerRef, className, position}: AnimationProps) => {

    const wrapperRef = React.useRef(null);
    const computedTriggerRef = triggerRef || wrapperRef;

    useLayoutEffect(() => {
        let ctx = gsap.context(() => {

            const tl = onScroll ? gsap.timeline({
                scrollTrigger: {
                    trigger: computedTriggerRef.current,
                    onEnter: () => {
                        gsap.set(wrapperRef.current, { visibility: "visible" });
                    }
                }
            }): gsap.timeline();

            if (!onScroll) {
                tl.set(wrapperRef.current, { visibility: "visible" });
            }

            const splitText = new SplitText(wrapperRef.current, { type: "lines" });
            const splitTextLines = splitText.lines;

            const animation = {
                y: "30",
                opacity: 0,
                duration: 1, 
                ease: "quint",
                stagger: { each: 0.03 },
            }

            tl.from(splitTextLines,animation, position || 0);
        
            tl.then(() => {
            splitText.revert()
            }) 
        });
        return () => {
            ctx.revert();
        }
    }, []);


    return (
        <div className={getStyle(className)}>
            <div className="invisible" ref={wrapperRef}>
                {children}
            </div>
        </div>
    )
}


