"use client";

import {
  PlotOptions,
  barY,
  lineY,
  areaY,
  binX,
  ruleY,
  groupX,
  barX,
} from "@observablehq/plot";
import React, { useState } from "react";
import PlotComponent from "./ui/plot-react";

interface Props {
  data: object[];
  toolbar?: React.ReactElement;
  yAccessor: string;
  xAccessor?: string;
  fillAccessor?: string;
}

function createChartOption(chartType: string) {
  switch (chartType) {
    case "line":
      return lineY;
    case "area":
      return areaY;
    case "bar":
      return barX;
    default:
      return lineY;
  }
}

function createDrawOption(chartType: string, breakdownKey: string) {
  switch (chartType) {
    case "line":
      return { stroke: breakdownKey };
    case "area":
      return { fill: breakdownKey };
    case "bar":
      return { fill: breakdownKey };
    default:
      return lineY;
  }
}

export default function Histogram({
  data,
  yAccessor,
  xAccessor = "time",
  fillAccessor,
}: Props) {
  const [chartType, setChartType] = useState("bar");
  console.log(data);
  const options: PlotOptions = {
    y: {
      grid: true,
    },
    marks: [
      createChartOption(chartType)(
        data,

        {
          y: yAccessor,

          x: "hits",
          ...createDrawOption(chartType, yAccessor),

          tip: { fill: "black" },
        }
      ),
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
