"use client"
import { Vega } from 'react-vega';
import { TopLevelSpec } from 'vega-lite'
const vl = require("vega-lite-api")
import { test } from './test'
import { ChartEditor } from 'components/chart-editor';
import { deserializeSpec } from 'lib/use-chart-binding';
import { useState } from 'react';
const nah: TopLevelSpec = {
    data: { values: test },
    mark: "circle",
    width: 500,
    height: 500,
    encoding: {
        "x": { "field": "Horsepower", "type": "quantitative" },
        "y": { "field": "Miles_per_Gallon", "type": "quantitative" },
        "color": { "field": "Origin", "type": "nominal" },
        "tooltip": { "field": "Name", "type": "nominal" },
    }

}

export default function Page() {
    const [spec, setSpec] = useState(nah)

    //  deserializeSpec(nah)




    return <div className=""><ChartEditor spec={nah} setSpec={setSpec} /><div>
        <Vega spec={spec} /></div></div>
}