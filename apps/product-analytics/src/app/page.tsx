"use client";
import { useEffect, useState } from "react";
import { DateRange } from "@/insights/time-query";
import { KPI, getKPI } from "@/insights/kpi";
import TimeSeriesChart from "@/components/time-series-chart";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { topBrowsersQuery } from "@/insights/top-browsers";
import Histogram from "@/components/histogram";
import { getData } from "./data";

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
  const [data, setData] = useState<KPI[]>([]);
  useEffect(() => {
    getKPI(DateRange["3D"]).then((val) => setData(val));
  }, []);

  console.log(data, "data");

  const [topBrowsers, setTopBrowsers] = useState([{}]);
  useEffect(() => {
    getData(topBrowsersQuery(DateRange["3D"])).then((val) =>
      setTopBrowsers(val),
    );
  }, []);

  return (
    <div className="m-2 h-screen w-screen grid gap-2 grid-cols-3 overflow-auto">
      <DashboardPanel name="Bounce Rate">
        <TimeSeriesChart
          yAccessor={"bounce_rate"}
          domain={[0, 100]}
          data={data}
          timeAccessor={(d) => new Date(d?.timestamp)}
        />
      </DashboardPanel>
      <DashboardPanel name="Avg Session Length (seconds)">
        <TimeSeriesChart
          yAccessor={"avg_session_sec"}
          data={data}
          timeAccessor={(d) => new Date(d?.timestamp)}
        />
      </DashboardPanel>
      <DashboardPanel name="Page Views">
        <TimeSeriesChart
          yAccessor={"pageviews"}
          data={data}
          timeAccessor={(d) => new Date(d?.timestamp)}
        />
      </DashboardPanel>
      <DashboardPanel name="Sessions">
        <TimeSeriesChart
          yAccessor={"visits"}
          data={data}
          timeAccessor={(d) => new Date(d?.timestamp)}
        />
      </DashboardPanel>
      <DashboardPanel name="Top Pages">
        <Histogram data={topBrowsers} yAccessor="pathname" />
      </DashboardPanel>
    </div>
  );
}
