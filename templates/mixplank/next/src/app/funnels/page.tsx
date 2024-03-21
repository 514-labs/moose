"use client"
import FunnelForm from "@/components/funnel-form";
import { createColumns } from "@/components/ui/data-table/columns";
import { DataTable } from "@/components/ui/data-table/data-table";
import { useEffect, useState } from "react";
import { getData } from "../data";
import { createFunnelQuery } from "@/insights/funnel-query";
import ReportLayout from "../report-layout";
import FunnelChart from "@/components/funnel-chart";
import { DateRange } from "@/insights/time-query";
import TimeSelector from "@/components/time-selector";



export default function Page() {
    const [formState, setFormState] = useState({ events: [] })
    const [data, setData] = useState<object[]>([]);
    const [dateRange, setDateRange] = useState(DateRange["30D"])

    useEffect(() => {
        getData(createFunnelQuery(formState, dateRange)).then(val => setData(val));
    }, [formState, dateRange])

    const table = data?.[0] && <DataTable columns={createColumns(data[0])} data={data} />;

    return (<ReportLayout chart={<FunnelChart toolbar={<TimeSelector selectedRange={dateRange} setDateRange={setDateRange} />} data={data} />} table={table} filterCard={<FunnelForm setForm={setFormState} />} />)

}