"use client"
import { useEffect, useState } from "react";
import ReportLayout from "../report-layout";
import { getData } from "../data";
import { tableQuery } from "@/insights/table-query";
import { DataTable } from "@/components/ui/data-table/data-table";
import { createColumns } from "@/components/ui/data-table/columns";
import { eventTables } from "@/insights/event-tables";
import TimeSeriesChart from "@/components/time-series-chart";
import TimeSelector from "@/components/time-selector";
import { DateRange } from "@/insights/time-query";
import TimeSeriesForm from "@/components/time-series-form";
import { FunnelFormList } from "@/components/funnel-form";
import { loadMixPanelData } from "@/lib/load-mixpanel-data";


export default function FunnelsPage() {
    const [formState, setFormState] = useState<FunnelFormList>({ events: [] })
    const [dateRange, setDateRange] = useState(DateRange["30D"])
    const [data, setData] = useState([{}]);
    useEffect(() => {
        const validForms = formState.events.filter((ev) => ev.eventName != null)
        const noEventsSelected = validForms.length == 0;
        const queriedEvents = noEventsSelected ? eventTables : validForms
        getData(tableQuery(queriedEvents, dateRange)).then(val => setData(val));
    }, [formState, dateRange])


    const newData = data.map((d) => ({ ...d, time: new Date(d?.time) }))

    const chart = <TimeSeriesChart toolbar={<TimeSelector selectedRange={dateRange} setDateRange={setDateRange} />} data={newData} />
    const table = data?.[0] && <DataTable columns={createColumns(data[0])} data={data} />



    return (
        <ReportLayout
            table={table}
            chart={chart}
            filterCard={<TimeSeriesForm setForm={setFormState} form={formState} />} />
    );
}
