"use client"

import { PlotOptions, ruleY, barY, lineY, binX, areaY } from "@observablehq/plot";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import React, { useState } from "react";
import PlotComponent from "./ui/plot-react";

interface Props {
    data: object[],
    toolbar: React.ReactElement;
}

const chartTypes = ['line', 'area', 'bar']

function createChartOption(chartType: string) {
    switch (chartType) {
        case 'line':
            return lineY;
        case 'area':
            return areaY;
        case 'bar':
            return barY;
        default:
            return lineY;
    }
}

function createDrawOption(chartType: string, breakdownKey: string) {
    switch (chartType) {
        case 'line':
            return { stroke: breakdownKey };
        case 'area':
            return { fill: breakdownKey };
        case 'bar':
            return { fill: breakdownKey };
        default:
            return lineY;
    }
}


export default function TimeSeriesChart({ data, toolbar }: Props) {
    const [chartType, setChartType] = useState('bar')

    const newData = data.map((d) => ({ ...d, time: new Date(d?.timestamp) }))

    const options: PlotOptions = {
        y: {
            grid: true,
        },
        color: { legend: true },
        marks: [
            createChartOption(chartType)(newData, { ...binX({ y: "count" }, { x: "time", ...createDrawOption(chartType, 'pathname'), interval: "minute" }), tip: { fill: "black" } }),
            ruleY([0]),
        ]
    }

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
                                <SelectItem
                                    key={i}
                                    value={type}
                                >
                                    {type}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
            </div>
            <div className="flex-1 m-4 overflow-hidden">

                {data?.[0] ? <PlotComponent options={options} /> : <div className="flex items-center justify-center h-full text-center">No Data in Time Range </div>}
            </div>
        </div>
    )
}

