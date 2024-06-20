"use client";

import Chart from "@/components/chart";
import QueryForm, { QueryFormData } from "@/components/query-form/query-form";
import TimeSelector from "@/components/time-selector";
import { createColumns } from "@/components/ui/data-table/columns";
import { DataTable } from "@/components/ui/data-table/data-table";
import { getMetric, getMetricList, getMetricTimeSeries } from "@/data-api";
import { DateRange } from "@/insights/time-query";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { SortingState } from "@tanstack/react-table";

export default function Page() {
  const { isLoading, isError, data, error } = useQuery({
    queryKey: [],
    queryFn: getMetricList,
    initialData: [],
  });
  const [orderBy, setOrderBy] = useState<SortingState>([]);
  const [selectedRange, setSelectedRange] = useState(DateRange["3D"]);
  const [formState, setFormState] = useState<QueryFormData[]>([]);
  const { data: metricData } = useQuery({
    queryKey: ["query", formState, selectedRange, orderBy],
    queryFn: () =>
      getMetric({
        query: formState[0],
        range: selectedRange,
        orderBy: orderBy,
      }),
    initialData: {},
  });

  const { data: chartData } = useQuery({
    queryKey: ["chart", formState, selectedRange],
    queryFn: () =>
      getMetricTimeSeries({ query: formState[0], range: selectedRange }),
    initialData: {},
  });

  if (isLoading) return <div>Loading...</div>;
  if (isError) return <div>Error: {error.message}</div>;

  const hasTable = metricData && metricData.length > 0;
  const hasChart = chartData && chartData.length > 0;

  return (
    <section className="flex-1 w-screen">
      <div className="w-full h-full grid grid-cols-3 grid-rows-3 gap-2 absolute">
        <div className="col-span-1 row-span-3">
          <QueryForm
            setOrderBy={setOrderBy}
            setForm={setFormState}
            name="Metric"
            options={data}
          />
        </div>
        <div className="row-span-1 col-span-2">
          <div className="w-fit ml-auto">
            <TimeSelector
              selectedRange={selectedRange}
              setDateRange={setSelectedRange}
            />
          </div>
          {hasChart && <Chart data={chartData} />}
        </div>
        <div className="col-span-2 row-span-2 overflow-auto">
          {hasTable && (
            <DataTable
              orderBy={orderBy}
              setOrderBy={setOrderBy}
              columns={createColumns(metricData[0])}
              data={metricData}
            />
          )}
        </div>
      </div>
    </section>
  );
}
