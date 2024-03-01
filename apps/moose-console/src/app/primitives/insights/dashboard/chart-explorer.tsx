"use client"
import { Vega } from 'react-vega';
import { TopLevelSpec } from 'vega-lite'
import { useForm } from 'react-hook-form'
import { Form } from 'components/ui/form'
import { ChartEditor, MarkForm } from 'components/chart-editor';
import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from 'components/ui/card';
import { Button } from 'components/ui/button'
import EventFilters from './event-filters';
import { Tabs, TabsList, TabsTrigger } from 'components/ui/tabs';
import { cn } from 'lib/utils';
import { segmentedTabListStyle, tabListStyle, tabTriggerStyle } from 'components/style-utils';
import { faker } from '@faker-js/faker';
interface ExplorerProps {
    data: object[]
}

/*
    useEffect(() => {
        // TypeScript users 
        const subscription = watch(() => handleSubmit(onSubmit)())
        return () => subscription.unsubscribe();
    }, [handleSubmit, watch]);
    const onSubmit = (data) => setSpec(data)

*/

export default function ChartExplorer(props: ExplorerProps) {
    const events = [...new Set(props.data.map(d => d["eventName"]))];

    const nah: TopLevelSpec = {
        data: { values: props.data },
        mark: "line",
        background: null,
        width: 'container',
        height: 'container',
        encoding: {
            "y": { "field": events[0], "type": "nominal", "aggregate": "count" },
            "x": { "field": "timestamp", "type": "temporal", "timeUnit": "day" },
            "color": { "field": "eventName", "type": "nominal" },
            "tooltip": { "field": "eventName", "type": "nominal" },
        }

    }
    const [spec, setSpec] = useState(nah)
    const methods = useForm({ defaultValues: spec })
    const { handleSubmit, watch } = methods;

    const onSubmit = (data) => setSpec(data)

    //  <EventFilters setSpec={setSpec} />

    //  <EventFilters spec={spec} setSpec={setSpec} fields={Object.keys(props.data[0])} />

    //                      <ChartEditor spec={nah} setSpec={setSpec} keys={keys} />
    return (
        <Form {...methods}>
            <form className={"h-full"} onSubmit={handleSubmit(onSubmit)}>
                <div className="grid grid-cols-2 w-full h-3/6 gap-4">

                    <Card>
                        <CardHeader>
                            <CardTitle>Filter by</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <EventFilters spec={spec} setSpec={setSpec} events={events} fields={Object.keys(props.data[0])} />

                        </CardContent>
                    </Card>
                    <Card>
                        <CardHeader>
                            <CardTitle className={"flex flex-row items-center justify-between align-middle"}>

                                <div className={"h-fit"}>Chart</div>
                                <div className='flex items-center'>
                                    <Tabs defaultValue='2W' onValueChange={(val) => console.log(val)}>
                                        <TabsList className={cn(segmentedTabListStyle, "flex-grow-0")}>
                                            <TabsTrigger className={cn(tabTriggerStyle, 'rounded-sm')} value="1D">
                                                1D
                                            </TabsTrigger>
                                            <TabsTrigger className={cn(tabTriggerStyle, 'rounded-sm')} value="3D">
                                                3D
                                            </TabsTrigger>
                                            <TabsTrigger className={cn(tabTriggerStyle, 'rounded-sm')} value="1W">
                                                1W
                                            </TabsTrigger>
                                            <TabsTrigger className={cn(tabTriggerStyle, 'rounded-sm')} value="2W">
                                                2W
                                            </TabsTrigger>
                                        </TabsList>
                                    </Tabs>
                                    <MarkForm options={['line', 'bar', 'circle', 'area']} spec={spec} setSpec={setSpec} />
                                </div>
                            </CardTitle>

                        </CardHeader>
                        <CardContent className='h-full'>
                            <Vega className={"w-full h-full"} theme='carbong100' spec={spec} actions={false} />
                        </CardContent>
                    </Card>
                </div>
            </form>
        </Form>
    );

}