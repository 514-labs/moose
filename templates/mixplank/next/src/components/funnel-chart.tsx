import { PlotOptions, barY, ruleY } from "@observablehq/plot"
import PlotComponent from "./ui/plot-react"

interface Props {
    data: object[],
    toolbar: React.ReactElement,
}
export default function FunnelChart({ data, toolbar }: Props) {
    const options: PlotOptions = {
        y: {
            grid: true,
        },
        marks: [
            barY(data, { y: "count", x: "level", tip: { fill: "black" } }),
            ruleY([0]),
        ]
    }

    return <div className="w-full h-full flex flex-col">
        <div className="h-12 justify-end flex w-full p-2">
            {toolbar}
        </div>
        <div className="flex-1 overflow-hidden m-4">

            {data?.[0] ? <PlotComponent options={options} /> : <div className="flex items-center justify-center h-full text-center">No Data in Time Range for selected events</div>}
        </div>
    </div>
}

