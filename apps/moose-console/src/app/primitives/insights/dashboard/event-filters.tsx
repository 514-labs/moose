import { TabsTrigger } from "@radix-ui/react-tabs";
import { tabListStyle, tabTriggerStyle } from "components/style-utils";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "components/ui/select";
import { Tabs, TabsContent, TabsList } from "components/ui/tabs";
import { cn } from "lib/utils";
import { Dispatch, SetStateAction } from "react";
import { TopLevelSpec } from "vega-lite";
import { TopLevelUnitSpec } from "vega-lite/build/src/spec/unit";


interface EventFilterProps {
    spec: TopLevelSpec;
    setSpec: Dispatch<SetStateAction<TopLevelSpec>>;
    fields: string[];
    events: string[]
}

function chartModifiers(spec: TopLevelUnitSpec<any>) {
    return {
        setMetric: (field: string): TopLevelSpec => ({
            ...spec,
            "transform": [{ "filter": { "field": "eventName", "equal": field } }],
        }),
        setBreakdown: (field: string): TopLevelSpec => ({
            ...spec,
            encoding: {
                ...spec.encoding,
                "color": { "field": field, "type": "nominal" },
            }
        })
    }
}

export default function EventFilters({ spec, setSpec, fields, events }: EventFilterProps) {
    const { setMetric, setBreakdown } = chartModifiers(spec)
    return <Tabs defaultValue="engagement">
        <TabsList className={cn(tabListStyle, "flex-grow-0")}>
            <TabsTrigger className={cn(tabTriggerStyle)} value="engagement">
                Engagement
            </TabsTrigger>
            <TabsTrigger className={cn(tabTriggerStyle)} value="funnel">
                Funnel
            </TabsTrigger>
        </TabsList>
        <TabsContent value="engagement">
            <div>
                <div>Metric</div>
            </div>
            <div><Select
                onValueChange={(value) => {
                    setSpec(setMetric(value))
                }}
            >
                <SelectTrigger>
                    <SelectValue />
                </SelectTrigger>
                <SelectContent>
                    {events.map((field, index) => (
                        <SelectItem
                            key={index}
                            value={field}
                        >
                            {field}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select></div>
            <div>
                <div>Breakdown</div>
            </div>
            <div><Select
                onValueChange={(value) => {
                    setSpec(setBreakdown(value))
                }}
            >
                <SelectTrigger>
                    <SelectValue />
                </SelectTrigger>
                <SelectContent>
                    {fields.map((field, index) => (
                        <SelectItem
                            key={index}
                            value={field}
                        >
                            {field}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select></div>

        </TabsContent>
        <TabsContent value="funnel">
            <div>Funnel</div>
        </TabsContent>
    </Tabs >
}