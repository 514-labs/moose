"use client"
import FunnelForm from "@/components/funnel-form";
import { Card, CardContent, CardDescription, CardHeader } from "@/components/ui/card";
import { createColumns } from "@/components/ui/data-table/columns";
import { DataTable } from "@/components/ui/data-table/data-table";
import PlotComponent from "@/components/ui/plot-react";
import { PlotOptions, ruleY, barY, lineY, binX, areaY} from "@observablehq/plot";
import { useEffect, useState } from "react";
import { getData } from "../data";
import { Button } from "@/components/ui/button";
import { useRouter } from "next/navigation";
import { eventTables } from "@/data/event-tables";
import { tableQuery } from "@/data/table-query";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";


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


export function FunnelChartPage() {
    const router = useRouter()
    const [formState, setFormState] = useState({ events: [] })
    const [data, setData] = useState([{}]);
    useEffect(() => {
        getData(tableQuery(eventTables)).then(val => setData(val));
    }, [formState])

    const [chartType, setChartType] = useState('bar')

    const newData = data.map((d) => ({ ...d, time: new Date(d?.time) }))

    const options: PlotOptions = {
        y: {
            grid: true,
        },
        color: { legend: true },
        marks: [
            createChartOption(chartType)(newData, { ...binX({ y: "count" }, { x: "time", ...createDrawOption(chartType, 'city'), interval: "day" }), tip: { fill: "black" } }),
            ruleY([0]),
        ]
    }

    return <div className="grid grid-rows-4 grid-cols-5 gap-4 h-screen w-full m-4">
        <Card className="row-span-2 col-span-2">
            <CardHeader>
                <CardDescription>Analysis Type</CardDescription>
                <div className="flex">
                <Button className="rounded-xl bg-accent mr-2" variant={"outline"} onClick={() => router.push('/insights')}>Engagement</Button>
                <Button className="rounded-xl" variant={"outline"} onClick={() => router.push('/funnels')}>Funnels</Button>
                </div>
            </CardHeader>
            <CardContent>
  
                
            </CardContent>
        </Card>
        <Card className="col-span-3 row-span-2">
            <CardHeader className="justify-end">
                <div className="w-36">
                    <Select value={chartType} onValueChange={(val) => setChartType(val)}>
                        <SelectTrigger className="rounded-xl">
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
            </CardHeader>
            {data?.[0] && <PlotComponent options={options} />}
        </Card>
        <Card className="row-span-2 col-span-5">
            {data?.[0] && <DataTable columns={createColumns(data[0])} data={data} />}
        </Card>
    </div>

}