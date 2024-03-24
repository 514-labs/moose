"use client";
import { useEffect, useState } from "react";
import { getData } from "../data";
import { DateRange } from "@/insights/time-query";
import { kpiQuery } from "@/insights/kpi";
import TimeSeriesChart from "@/components/time-series-chart";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { topBrowsersQuery } from "@/insights/top-browsers";
import Histogram from "@/components/histogram";

const DashboardPanel = ({
  name,
  children,
}: {
  name: string;
  children: React.ReactElement;
}) => {
  return (
    <Card className="h-96 w-full flex flex-col">
      <CardHeader>
        <CardTitle>{name}</CardTitle>
      </CardHeader>
      <CardContent className="h-full w-full p-4 m-0">{children}</CardContent>
    </Card>
  );
};

export default function DashboardPage() {
  const [data, setData] = useState([{}]);
  useEffect(() => {
    getData(kpiQuery(DateRange["7D"])).then((val) => setData(val));
  }, []);
  const [topBrowsers, setTopBrowsers] = useState([{}]);
  useEffect(() => {
    getData(topBrowsersQuery(DateRange["7D"])).then((val) =>
      setTopBrowsers(val)
    );
  }, []);
  console.log(data);
  return (
    <div className="m-2 h-screen w-screen grid gap-2 grid-cols-2 overflow-auto">
      <DashboardPanel name="Bounce Rate">
        <TimeSeriesChart
          yAccessor={"bounce_rate"}
          data={data}
          timeAccessor={(d) => new Date(d?.date)}
        />
      </DashboardPanel>
      <DashboardPanel name="Avg Session Length (seconds)">
        <TimeSeriesChart
          yAccessor={"avg_session_sec"}
          data={data}
          timeAccessor={(d) => new Date(d?.date)}
        />
      </DashboardPanel>
      <DashboardPanel name="Page Views">
        <TimeSeriesChart
          yAccessor={"pageviews"}
          data={data}
          timeAccessor={(d) => new Date(d?.date)}
        />
      </DashboardPanel>
      <DashboardPanel name="Sessions">
        <TimeSeriesChart
          yAccessor={"visits"}
          data={data}
          timeAccessor={(d) => new Date(d?.date)}
        />
      </DashboardPanel>
      <DashboardPanel name="Top Pages">
        <Histogram data={topBrowsers} yAccessor="pathname" />
      </DashboardPanel>
    </div>
  );
}
