"use client";

import { PlotOptions, lineY, areaY, rectY } from "@observablehq/plot";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import React, { useState } from "react";
import PlotComponent from "./ui/plot-react";
import { TimeUnit } from "@/lib/time-utils";

interface Props {
  data: object & { timestamp: string }[];
  toolbar?: React.ReactElement;
  timeAccessor: (arr: object & { timestamp: string }) => Date;
  yAccessor: string;
  fillAccessor: ((arr: object) => string) | string;
  interval: TimeUnit;
}

const chartTypes = ["line", "area", "bar"];

function createChartOption(chartType: string) {
  switch (chartType) {
    case "line":
      return lineY;
    case "area":
      return areaY;
    case "bar":
      return rectY;
    default:
      return lineY;
  }
}

function createDrawOption(
  chartType: string,
  breakdownKey: ((arr: object) => string) | string,
) {
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
export default function HistogramChart({
  data,
  toolbar,
  timeAccessor,
  yAccessor,
  fillAccessor,
  interval,
}: Props) {
  const [chartType, setChartType] = useState("bar");

  const newData = data.map((d) => ({ ...d, time: timeAccessor(d) }));

  const options: PlotOptions = {
    color: {
      scheme: "Plasma",
      range: [0.2, 0.8],
    },
    y: {
      grid: true,
    },
    marks: [
      createChartOption(chartType)(newData, {
        y: yAccessor,
        x: "time",
        interval: interval,
        tip: { fill: "black" },
        ...createDrawOption(chartType, fillAccessor),
      }),
    ],
  };

  return (
    <div className="w-full h-full flex flex-col">
      <div className="h-12 justify-end flex w-full p-2">
        {toolbar}
        <div className="w-36">
          <Select value={chartType} onValueChange={(val) => setChartType(val)}>
            <SelectTrigger className="rounded-xl capitalize">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {chartTypes.map((type, i) => (
                <SelectItem key={i} value={type}>
                  {type}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
      <div className="flex-1 m-4 overflow-hidden">
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
