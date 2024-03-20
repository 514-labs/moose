"use client"
import FunnelForm from "@/components/funnel-form";
import { Card, CardContent, CardDescription, CardHeader } from "@/components/ui/card";
import { createColumns } from "@/components/ui/data-table/columns";
import { DataTable } from "@/components/ui/data-table/data-table";
import PlotComponent from "@/components/ui/plot-react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { PlotOptions, ruleY, barY } from "@observablehq/plot";
import { useEffect, useState } from "react";
import { getData } from "./data";
import { createFunnelQuery } from "@/insights/funnel-query";



export function ChartPage() {
    const [formState, setFormState] = useState({ events: [] })
    const [data, setData] = useState();
    useEffect(() => {
        getData(createFunnelQuery(formState)).then(val => setData(val));
    }, [formState])

    const options: PlotOptions = {
        y: {
            grid: true,
        },
        marks: [
            barY(data, { y: "count", x: "level" }),
            ruleY([0])
        ]
    }

    return <div className="grid grid-rows-4 grid-cols-5 gap-4 h-screen w-full">
        <Card className="row-span-2 col-span-2">
            <CardHeader>
                <CardDescription>Analysis Type</CardDescription>
            </CardHeader>
            <CardContent>
                <Tabs defaultValue="engagement">
                    <TabsList>
                        <TabsTrigger value="engagement">
                            Engagement
                        </TabsTrigger>
                        <TabsTrigger value="funnel">
                            Funnel
                        </TabsTrigger>
                    </TabsList>
                    <TabsContent value="funnel">
                        <FunnelForm setForm={setFormState} />
                    </TabsContent>
                </Tabs>
            </CardContent>
        </Card>
        <Card className="col-span-3 row-span-2">{data?.[0] && <PlotComponent options={options} />}</Card>
        <Card className="row-span-2 col-span-5">{data?.[0] && <DataTable columns={createColumns(data[0])} data={data} />}</Card>
    </div>

}