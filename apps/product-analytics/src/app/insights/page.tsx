"use client";
import { useEffect, useState } from "react";
import ReportLayout from "../report-layout";
import { getTableQueryData, getChartQueryData } from "@/insights/table-query";
import { DataTable } from "@/components/ui/data-table/data-table";
import { createColumns } from "@/components/ui/data-table/columns";
import TimeSelector from "@/components/time-selector";
import { DateRange } from "@/insights/time-query";
import TimeSeriesForm from "@/components/time-series-form";
import HistogramChart from "@/components/histogram-chart";
import { ModelMeta, getModelMeta } from "@/insights/model-meta";
import { TimeUnit } from "@/lib/time-utils";
import { eventConfigFromNames } from "@/app/events";
import { MetricForm } from "@/lib/form-types";

function defaultBreakdown(breakdown: string[]) {
  return breakdown.filter((b) => b != null).length == 0 ? [] : breakdown;
}

export default function InsightsPage() {
  const [formState, setFormState] = useState<MetricForm[]>([]);
  const [dateRange, setDateRange] = useState(DateRange.Today);
  const [breakdown, setBreakdown] = useState<string[]>([]);
  const [interval, setInterval] = useState<TimeUnit>(TimeUnit.HOUR);
  const [data, setData] = useState([{}]);
  const [timeSeries, setTimeSeries] = useState<
    object & { timestamp: string }[]
  >([]);
  const [modelInfo, setModelMeta] = useState<ModelMeta[]>([]);
  const defaultedBreakdown = defaultBreakdown(breakdown);

  useEffect(() => {
    const defaultedBreakdown = defaultBreakdown(breakdown);

    const queriedEvents = eventConfigFromNames(formState);
    getTableQueryData(queriedEvents, dateRange, defaultedBreakdown).then(
      (val) => setData(val),
    );
    getChartQueryData(
      queriedEvents,
      dateRange,
      interval,
      defaultedBreakdown,
    ).then((val) => setTimeSeries(val));
  }, [formState, dateRange, breakdown, interval]);

  useEffect(() => {
    const queriedEvents = eventConfigFromNames(formState);
    if (modelInfo) {
      getModelMeta(queriedEvents).then((val) => setModelMeta(val));
    }
  }, [formState]);

  const chart = (
    <HistogramChart
      timeAccessor={(obj: object & { timestamp: string }) =>
        new Date(obj?.timestamp)
      }
      yAccessor="count"
      fillAccessor={(d: { [key: string]: any }) => {
        return defaultedBreakdown.map((b) => d[b]).join(", ");
      }}
      interval={interval}
      toolbar={
        <TimeSelector
          interval={interval}
          setInterval={setInterval}
          selectedRange={dateRange}
          setDateRange={setDateRange}
        />
      }
      data={timeSeries}
    />
  );
  const table = data?.[0] && (
    <DataTable columns={createColumns(data[0])} data={data} />
  );
  return (
    <div>
      <ReportLayout
        table={table}
        chart={chart}
        filterCard={
          <TimeSeriesForm
            breakdownOptions={modelInfo}
            setBreakdown={setBreakdown}
            setForm={setFormState}
          />
        }
      />
    </div>
  );
}
