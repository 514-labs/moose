"use client";

import Chart from "@/components/chart";
import QueryForm, { QueryFormData } from "@/components/query-form/query-form";
import { createColumns } from "@/components/ui/data-table/columns";
import { DataTable } from "@/components/ui/data-table/data-table";
import { getMetric, getMetricList, getMetricTimeSeries } from "@/data-api";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

export default function Page() {
  const { isLoading, isError, data, error } = useQuery({
    queryKey: [],
    queryFn: getMetricList,
    initialData: [],
  });
  const [formState, setFormState] = useState<QueryFormData[]>([]);
  const {
    isLoading: isMetricLoading,
    isError: isMetricError,
    data: metricData,
    error: metricError,
  } = useQuery({
    queryKey: ["query", formState],
    queryFn: () => getMetric({ query: formState[0] }),
    initialData: {},
  });

  const {
    isLoading: isChartLoading,
    isError: isChartError,
    data: chartData,
    error: chartError,
  } = useQuery({
    queryKey: ["chart", formState],
    queryFn: () => getMetricTimeSeries({ query: formState[0] }),
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
          <QueryForm setForm={setFormState} name="Metric" options={data} />
        </div>
        <div className="row-span-1 col-span-2">
          {hasChart && <Chart data={chartData} />}
        </div>
        <div className="col-span-2 row-span-2 overflow-auto">
          {hasTable && (
            <DataTable
              columns={createColumns(metricData[0])}
              data={metricData}
            />
          )}
        </div>
      </div>
    </section>
  );
}
