"use client";

import { PlotOptions, areaY, axisX, axisY } from "@observablehq/plot";
import React from "react";
import PlotComponent from "./ui/plot-react";
import { interpolatePlasma } from "d3";

interface Props {
  data: object & { timestamp: string }[];
  toolbar?: React.ReactElement;
  timeAccessor: (arr: object & { timestamp: string }) => Date;
  yAccessor: string;
  xAccessor?: string;
  fillAccessor?: string;
  domain?: [number, number];
  percent?: boolean;
}

export default function TimeSeriesChart({
  data,
  timeAccessor,
  yAccessor,
  xAccessor = "time",
  domain,
  percent,
}: Props) {
  const newData = data.map((d) => ({ ...d, time: timeAccessor(d) }));

  const barColor = interpolatePlasma(0.3);

  const options: PlotOptions = {
    y: {
      ...(domain && { domain }),
      percent: percent,
    },
    axis: null,
    marks: [
      areaY(newData, {
        x: xAccessor,
        y: yAccessor,
        interval: "hour",
        fill: barColor,
        tip: { fill: "black" },
      }),
      axisY({ label: null }),
      axisX({ ticks: "day" }),
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
