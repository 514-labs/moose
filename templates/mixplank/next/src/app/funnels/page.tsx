"use client";
import { createColumns } from "@/components/ui/data-table/columns";
import { DataTable } from "@/components/ui/data-table/data-table";
import { useEffect, useState } from "react";
import { getData } from "../data";
import { createFunnelQuery } from "@/insights/funnel-query";
import ReportLayout from "../report-layout";
import FunnelChart from "@/components/funnel-chart";
import { DateRange } from "@/insights/time-query";
import TimeSelector from "@/components/time-selector";
import TimeSeriesForm from "@/components/time-series-form";
import { eventConfigFromNames } from "@/config";
import { ModelMeta, getModelMeta } from "@/insights/model-meta";
import { MetricForm } from "@/lib/form-types";

export default function Page() {
  const [formState, setFormState] = useState<MetricForm[]>([]);
  const [data, setData] = useState<object[]>([]);
  const [dateRange, setDateRange] = useState(DateRange["7D"]);
  const [modelInfo, setModelMeta] = useState<ModelMeta[]>([]);
  const [breakdown, setBreakdown] = useState<string[]>([]);

  useEffect(() => {
    const queriedEvents = eventConfigFromNames(formState);
    getData(createFunnelQuery(queriedEvents, dateRange, breakdown)).then(
      (val) => setData(val)
    );
  }, [formState, dateRange, breakdown]);

  useEffect(() => {
    const queriedEvents = eventConfigFromNames(formState);
    if (modelInfo && queriedEvents.length > 0) {
      getModelMeta(queriedEvents).then((val) => setModelMeta(val));
    }
  }, [formState]);

  const table = data?.[0] && (
    <DataTable columns={createColumns(data[0])} data={data} />
  );

  return (
    <ReportLayout
      chart={
        <FunnelChart
          breakdown={breakdown}
          toolbar={
            <TimeSelector
              selectedRange={dateRange}
              setDateRange={setDateRange}
            />
          }
          data={data}
        />
      }
      table={table}
      filterCard={
        <TimeSeriesForm
          breakdownOptions={modelInfo}
          setBreakdown={setBreakdown}
          setForm={setFormState}
        />
      }
    />
  );
}
