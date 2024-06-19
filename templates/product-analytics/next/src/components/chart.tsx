"use client";

import * as d3 from "d3";
import { useEffect, useRef, useState } from "react";

export type TimeSeriesPoint = [string, string];

interface Props {
  data: { timeseries: TimeSeriesPoint[] }[];
}

export default function Chart({ data }: Props) {
  const [height, setHeight] = useState(0);
  const [width, setWidth] = useState(0);

  const ref = useRef<HTMLDivElement>(null);

  const timeseries = data[0].timeseries;

  useEffect(() => {
    if (!ref.current) {
      return;
    }

    const resizeObserver = new ResizeObserver(() => {
      if (ref.current && ref.current.offsetHeight !== height) {
        setHeight(ref.current.offsetHeight);
      }
      if (ref.current && ref.current.offsetWidth !== width) {
        setWidth(ref.current.offsetWidth);
      }
    });
    resizeObserver.observe(ref.current);
    return function cleanup() {
      resizeObserver.disconnect();
    };
  }, [ref.current]);

  const chartRef = useRef();

  const margin = { top: 16, right: 16, bottom: 50, left: 30 },
    chartWidth = width - margin.left - margin.right,
    chartHeight = height - margin.top - margin.bottom;

  const timeExtent = d3.extent(timeseries, (d) => d[0]) as [Date, Date];
  const yExtent = d3.extent(timeseries, (d) => d[1]) as [number, number];
  const x = d3.scaleTime().domain(timeExtent).range([0, chartWidth]);
  const y = d3.scaleLinear().domain(yExtent).range([chartHeight, 0]);

  useEffect(() => {
    const svg = d3.select(chartRef.current);

    // Remove old elements before appending new ones
    svg.select(".x-axis").remove();
    svg.select(".y-axis").remove();
    svg.select(".x-label").remove();
    svg.select(".y-label").remove();

    // Append new elements
    svg
      .append("g")
      .attr("class", "x-axis")
      .attr("transform", `translate(0,${chartHeight})`)
      .call(d3.axisBottom(x));

    svg.append("g").attr("class", "y-axis").call(d3.axisLeft(y));
  }, [chartHeight, chartWidth, height, width]);

  useEffect(() => {
    // append the svg object to the body of the page

    const svg = d3.select(chartRef.current);

    svg.selectAll(".line").remove();
    svg.selectAll(".tooltip").remove();

    const color = d3
      .scaleLinear()
      .domain([0, data.length])
      .range(d3.schemeTableau10);

    const line = svg
      .selectAll(".line")
      .data(data)
      .enter()
      .append("g")
      .attr("fill", "none")
      .attr("stroke", (_d, i) => color(i))
      .on("mouseover", function () {
        d3.select(this).attr("stroke", "red").raise();
      })
      .on("mouseleave", function () {
        d3.select(this).attr("stroke", (_d, i) => color(i));
      })
      .attr("class", "line");

    line
      .selectAll("path")
      .data((d) => [d.timeseries])
      .join(
        (enter) => {
          return enter
            .append("path")
            .attr("class", "line")
            .attr("stroke-width", 1.5)
            .attr(
              "d",
              d3
                .line()
                .x((d) => {
                  return x(d[0]);
                })
                .y((d) => y(d[1])),
            );
        },
        (update) =>
          update.attr(
            "d",
            d3
              .line()
              .x((d) => x(d[0]))
              .y((d) => y(d[1])),
            (exit) => exit.remove(),
          ),
      );

    // tooltip

    const bisect = d3.bisector(function (d) {
      return d[0];
    }).left;

    const focus = svg
      .append("g")
      .append("line")
      .attr("y1", 0)
      .attr("y2", chartHeight)
      .style("stroke", "grey")
      .style("stroke-width", 1)
      .style("stroke-dasharray", "3,3")

      .style("fill", "none")
      .attr("stroke", "grey")
      .style("opacity", 0);

    const focusTextBackground = svg
      .append("g")
      .append("rect")
      .style("opacity", 0)
      .attr("fill", "lightblue")
      .attr("width", 75)
      .attr("height", 25);

    const focusText = svg
      .append("g")
      .append("text")
      .style("opacity", 0)
      .attr("text-anchor", "left")
      .attr("alignment-baseline", "middle");

    // Update the position and size of the background rect when the text is updated
    /*
    svg
      .append("rect")
      .style("fill", "none")
      .style("pointer-events", "all")
      .attr("width", width)
      .attr("height", height)
      .on("mouseover", mouseover)
      .on("mousemove", mousemove)
      .on("mouseout", mouseout);
      */

    // What happens when the mouse move -> show the annotations at the right positions.
    function mouseover() {
      focus.style("opacity", 1);
      focusText.style("opacity", 1);
      focusTextBackground.style("opacity", 1);
    }

    function mousemove(event) {
      // recover coordinate we need
      var x0 = x.invert(d3.pointer(event)[0]);
      var i = bisect(timeseries, x0, 1);
      let selectedData = timeseries[i];
      if (selectedData) {
        focus.attr("x1", x(selectedData[0])).attr("x2", x(selectedData[0]));
        focusTextBackground.attr("x", x(selectedData[0]) - 37.5).attr("y", -25);

        focusText
          .html(`Val: ${selectedData[1]}}`)
          .attr("x", x(selectedData[0]) - 37.5)
          .attr("y", -10);
      }
    }

    function mouseout() {
      focus.style("opacity", 0);
      focusText.style("opacity", 0);
      focusTextBackground.style("opacity", 0);
    }
  }, [timeseries, height, width]);
  return (
    <div ref={ref} className="h-full w-full">
      <svg width={width} height={height} id="line_chart">
        <g
          ref={chartRef}
          transform={`translate(${margin.left},${margin.top})`}
        ></g>
      </svg>
    </div>
  );
}
