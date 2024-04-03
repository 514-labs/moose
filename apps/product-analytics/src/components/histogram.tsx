"use client";

import { PlotOptions, ruleY, barY, barX } from "@observablehq/plot";
import React from "react";
import PlotComponent from "./ui/plot-react";

interface Props {
  data: object[];
  toolbar?: React.ReactElement;
  yAccessor: string;
  xAccessor?: string;
  fillAccessor?: string;
}

export default function Histogram({ data, yAccessor }: Props) {
  const options: PlotOptions = {
    y: {
      grid: true,
    },
    marks: [
      barX(data, {
        y: yAccessor,
        x: "hits",
        fill: yAccessor,
        tip: { fill: "black" },
      }),
      ruleY([0]),
    ],
  };

  return (
    <div className="w-full h-full flex flex-col">
      <div className="flex-1 overflow-hidden">
        {data?.[0] ? (
          <PlotComponent options={options} />
        ) : (
          <div className="flex items-center justify-center h-full text-center">
            No Data in Time Range{" "}
          </div>
        )}
      </div>
    </div>
  );
}
