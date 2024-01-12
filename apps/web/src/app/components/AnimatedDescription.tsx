'use client'

import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);

gsap.registerPlugin(SplitText);


interface DescriptionProps {
    content: string,
    onScroll?: boolean, // Triggers when the user scrolls to the element
    triggerRef?: React.MutableRefObject<HTMLDivElement>, // The ref to the element that triggers the animation. Defaults to the element itself
    className?: string,
    position?: number,
}

const getStyle = (className: string) => {
    if (className) {
        return className + "text-typography my-3 text-black"
    } else {
        return "text-typography my-3 text-black";
    }
}


export const AnimatedDescription = ({content, onScroll, triggerRef, className, position}: DescriptionProps) => {

    const descriptionRef = React.useRef(null);
    const computedTriggerRef = triggerRef || descriptionRef;
    var computedPosition = position || 0;

    useLayoutEffect(() => {
        const splitText = new SplitText(descriptionRef.current, { type: "lines, words" });
        const splitTextLines = splitText.lines;

        const resetSplitText = () => {
            splitText.revert();
        }

        window.addEventListener("resize", resetSplitText);

        let ctx = gsap.context(() => {

            const tl = onScroll ? gsap.timeline({
                scrollTrigger: {
                    trigger: computedTriggerRef.current,
                    onEnter: (self) => {
                        gsap.set(descriptionRef.current, { visibility: "visible" });
                        if (self.getVelocity() > 0) {
                            computedPosition = 0;
                        }
                    }
                },
            }): gsap.timeline();
            if (!onScroll) {
                tl.set(descriptionRef.current, { visibility: "visible" });
            }

            const animation = {
                y: "20",
                opacity: 0,
                duration: 1,
                ease: "quint",
                stagger: { each: 0.04 },
            }

            tl.from(splitTextLines,animation, position || 0);
            tl.then(() => {
                splitText.revert();
            })
        });
        return () => {
            window.addEventListener("resize", resetSplitText);
            ctx.revert();
        }
    }, [descriptionRef, computedTriggerRef]);


    return (
        <div className={getStyle(className)}>
            <div className="invisible" ref={descriptionRef}>
                {content}
            </div>
        </div>
    )
}


