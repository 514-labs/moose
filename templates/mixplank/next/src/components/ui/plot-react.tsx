"use client"

import * as Plot from "@observablehq/plot";
import { useEffect, useRef, useState } from "react";


interface Data {
    options: Plot.PlotOptions
}
export default function PlotComponent({ options }: Data) {
    const [height, setHeight] = useState(0)
    const [width, setWidth] = useState(0)

    const ref = useRef(null)

    useEffect(() => {
        if (!ref.current) {
            return;
        }

        const resizeObserver = new ResizeObserver(() => {
            if (ref.current.offsetHeight !== height) {
                setHeight(ref.current.offsetHeight);
            }
            if (ref.current.offsetWidth !== width) {
                setWidth(ref.current.offsetWidth);
            }
        });
        resizeObserver.observe(ref.current);
        return function cleanup() {
            resizeObserver.disconnect();
        }
    },
        [ref.current])

    const containerRef = useRef<HTMLDivElement>(null);
    useEffect(() => {
        // -24 to offset legend height, can make this less brittle later
        const newOptions = { ...options, height: height - 24, width: width }
        const plot = Plot.plot(newOptions);
        containerRef.current?.append(plot);
        return () => plot.remove();
    }, [options, height]);

    return <div ref={ref} className="h-full w-full"><div ref={containerRef} /></div>;
}