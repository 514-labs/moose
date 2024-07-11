"use client";

import * as React from "react";
import { Bar, BarChart, CartesianGrid, XAxis } from "recharts";

import { Card, CardContent } from "@/components/ui/card";
import {
  ChartConfig,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart";
import { SeverityLevel } from "@/lib/utils";
import { useQuery } from "@tanstack/react-query";
import { fetchLogTimeseries } from "@/lib/data-fetch";

const chartConfig = {
  views: {
    label: "Logs",
  },
  warn: {
    label: "warn",
    color: "hsl(var(--chart-1))",
  },
  error: {
    label: "error",
    color: "hsl(var(--chart-2))",
  },
} satisfies ChartConfig;

interface Props {
  source: string | undefined;
  search: string;
  severity: SeverityLevel[];
}
export default function StackedBar({ source, search, severity }: Props) {
  const { data: logTimeSeries, isFetched } = useQuery({
    queryKey: ["timeseries", search, severity, source],
    queryFn: () => fetchLogTimeseries({ search, severity, source }),
    initialData: [],
  });

  return (
    <Card className="bg-background">
      <CardContent className="px-2 sm:p-6">
        <ChartContainer
          config={chartConfig}
          className="aspect-auto h-[250px] w-full"
        >
          <BarChart
            accessibilityLayer
            data={logTimeSeries}
            margin={{
              left: 12,
              right: 12,
            }}
          >
            <CartesianGrid vertical={false} />
            <XAxis
              dataKey="date"
              tickLine={false}
              axisLine={false}
              tickMargin={8}
              minTickGap={32}
              tickFormatter={(value) => {
                const date = new Date(value);
                return date.toLocaleDateString("en-US", {
                  month: "short",
                  day: "numeric",
                });
              }}
            />
            <ChartTooltip
              content={
                <ChartTooltipContent
                  className="w-[150px]"
                  labelFormatter={(value) => {
                    return new Date(value).toLocaleDateString("en-US", {
                      month: "short",
                      day: "numeric",
                      year: "numeric",
                      hour: "numeric",
                    });
                  }}
                />
              }
            />
            <Bar dataKey={"warn"} stackId={"a"} fill={"#bfdbfe"} />
            <Bar dataKey={"error"} stackId={"a"} fill={`#fecaca`} />
          </BarChart>
        </ChartContainer>
      </CardContent>
    </Card>
  );
}
