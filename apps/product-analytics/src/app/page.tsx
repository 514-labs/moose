"use client";
import { useEffect, useState } from "react";
import { DateRange } from "@/insights/time-query";
import { KPI, getKPI } from "@/insights/kpi";
import TimeSeriesChart from "@/components/time-series-chart";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { topBrowsersQuery } from "@/insights/top-browsers";
import Histogram from "@/components/histogram";
import { getData } from "./data";
import { Text, Heading, Display } from "design-system/typography";

const DashboardPanel = ({
  name,
  children,
}: {
  name: string;
  children: React.ReactElement;
}) => {
  return (
    <Card className="h-72 w-full flex flex-col rounded-2xl">
      <CardHeader className="p-4">
        <Text className="text-primary my-0">{name}</Text>
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

  const [topBrowsers, setTopBrowsers] = useState([{}]);
  useEffect(() => {
    getData(topBrowsersQuery(DateRange["3D"])).then((val) =>
      setTopBrowsers(val),
    );
  }, []);

  return (
    <div className="p-5 w-screen grid gap-5 grid-cols-3 overflow-auto p-0">
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
