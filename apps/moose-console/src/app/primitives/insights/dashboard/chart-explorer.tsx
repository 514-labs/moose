"use client"
import { Vega } from 'react-vega';
import { TopLevelSpec } from 'vega-lite'
import { useForm } from 'react-hook-form'
import { useEffect } from 'react'
import { Form, FormControl, FormField, FormItem, FormLabel } from 'components/ui/form'
const vl = require("vega-lite-api")
import { ChartEditor, MarkForm } from 'components/chart-editor';
import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from 'components/ui/card';

interface ExplorerProps {
    data: object[]
}



export default function ChartExplorer(props: ExplorerProps) {
    const nah: TopLevelSpec = {
        data: { values: props.data },
        mark: "circle",
        background: null,
        width: 'container',
        height: 'container',
        encoding: {
            "y": { "field": "eventType", "type": "nominal", "aggregate": "count" },
            "x": { "field": "timestamp", "type": "temporal", timeUnit: "day" },
            "color": { "field": "eventType", "type": "nominal" },
            "tooltip": { "field": "eventType", "type": "nominal", "aggregate": "count" },
        }

    }
    const [spec, setSpec] = useState(nah)
    const methods = useForm({ defaultValues: spec })
    const { handleSubmit, watch } = methods;
    useEffect(() => {
        // TypeScript users 
        // const subscription = watch(() => handleSubmit(onSubmit)())
        const subscription = watch(handleSubmit(onSubmit));
        return () => subscription.unsubscribe();
    }, [handleSubmit, watch]);
    const onSubmit = (data) => setSpec(data)

    const keys = Object.keys(props.data[0])

    //  deserializeSpec(nah)




    return (
        <Form {...methods}>
            <form className={"h-full"} onSubmit={handleSubmit(onSubmit)}>
                <div className="grid grid-cols-2 w-full h-3/6 gap-4">

                    <Card>
                        <CardHeader>
                            <CardTitle>Filter by</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <ChartEditor spec={nah} setSpec={setSpec} keys={keys} />
                        </CardContent>
                    </Card>
                    <Card>
                        <CardHeader>
                            <CardTitle className={"flex flex-row justify-between align-middle"}>
                                <div className={"h-fit"}>Chart</div>
                                <MarkForm options={['line', 'bar', 'circle', 'area']} formPath={'mark'} name={""} />
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