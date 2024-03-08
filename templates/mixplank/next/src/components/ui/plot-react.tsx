"use client"

import * as Plot from "@observablehq/plot";
import { useEffect, useRef } from "react";


interface Data {
    options: Plot.PlotOptions
}
export default function PlotComponent({ options }: Data) {
    const containerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const plot = Plot.plot(options);
        containerRef.current?.append(plot);
        return () => plot.remove();
    }, [options]);

    return <div ref={containerRef} />;
}