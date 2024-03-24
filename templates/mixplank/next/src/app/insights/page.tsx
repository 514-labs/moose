"use client";
import { useEffect, useState } from "react";
import ReportLayout from "../report-layout";
import { getData } from "../data";
import { tableQuery, testFunc, timeSeriesData } from "@/insights/table-query";
import { DataTable } from "@/components/ui/data-table/data-table";
import { createColumns } from "@/components/ui/data-table/columns";
import { eventTables } from "@/insights/event-tables";
import TimeSelector from "@/components/time-selector";
import { DateRange } from "@/insights/time-query";
import TimeSeriesForm from "@/components/time-series-form";
import { FunnelFormList } from "@/components/funnel-form";
import HistogramChart from "@/components/histogram-chart";

export default function InsightsPage() {
  const [formState, setFormState] = useState<FunnelFormList>({ events: [] });
  const [dateRange, setDateRange] = useState(DateRange["30D"]);
  const [data, setData] = useState([{}]);
  const [timeSeries, setTimeSeries] = useState();

  useEffect(() => {
    const validForms = formState.events.filter((ev) => ev.eventName != null);
    const noEventsSelected = validForms.length == 0;
    const queriedEvents = noEventsSelected ? eventTables : validForms;
    getData(tableQuery(queriedEvents, dateRange)).then((val) => setData(val));
  }, [formState, dateRange]);

  useEffect(() => {
    getData(testFunc(dateRange)).then((val) => setTimeSeries(val));
  }, [dateRange]);
  console.log(timeSeries);
  const chart = (
    <HistogramChart
      timeAccessor={(obj) => new Date(obj?.timestamp)}
      yAccessor="pathname"
      fillAccessor="pathname"
      toolbar={
        <TimeSelector selectedRange={dateRange} setDateRange={setDateRange} />
      }
      data={data}
    />
  );
  const table = data?.[0] && (
    <DataTable columns={createColumns(data[0])} data={data} />
  );

  return (
    <ReportLayout
      table={table}
      chart={chart}
      filterCard={<TimeSeriesForm setForm={setFormState} form={formState} />}
    />
  );
}
