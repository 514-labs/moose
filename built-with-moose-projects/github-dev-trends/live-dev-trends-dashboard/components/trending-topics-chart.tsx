"use client";

import { useState, useEffect } from "react";
import { CartesianGrid, XAxis, YAxis, BarChart, Bar } from "recharts";
import { Loader2 } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import mooseClient from "@/lib/moose-client";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart";
import { TrendingTopicsControls } from "./trending-topics-controls";

// Move color generation outside component to be deterministic
const PRESET_COLORS = [
  "#FF6B6B",
  "#4ECDC4",
  "#45B7D1",
  "#96CEB4",
  "#FFEEAD",
  "#D4A5A5",
  "#9B59B6",
  "#3498DB",
  "#1ABC9C",
  "#F1C40F",
];

const getTopicColor = (topic: string): string => {
  // Use hash of topic name to pick a consistent color
  const hash = topic
    .split("")
    .reduce((acc, char) => acc + char.charCodeAt(0), 0);
  return PRESET_COLORS[hash % PRESET_COLORS.length];
};

export function TrendingTopicsChart() {
  const [currentTimeIndex, setCurrentTimeIndex] = useState(0);
  const [interval, setInterval] = useState("hour");
  const [limit, setLimit] = useState(10);
  const [exclude, setExclude] = useState("");

  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ["topicTimeseries", interval, limit, exclude],
    queryFn: async () => {
      const result = await mooseClient.consumptionTopicTimeseriesGet({
        interval,
        limit,
        exclude,
      });
      return result;
    },
  });

  // Animation logic
  useEffect(() => {
    if (!data) return;

    const intervalId = window.setInterval(() => {
      setCurrentTimeIndex((prev) => (prev + 1) % data.length);
    }, 2000);

    return () => window.clearInterval(intervalId);
  }, [data]);

  if (isLoading && !data) {
    return (
      <div className="flex justify-center items-center h-80">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
        <span className="ml-2">Loading trending topics...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex justify-center items-center h-80 text-red-500">
        <p>
          Error loading data:{" "}
          {error instanceof Error ? error.message : "An unknown error occurred"}
        </p>
      </div>
    );
  }

  if (!data) return null;

  const chartConfig = Array.from(
    new Set(data.flatMap((d) => d.topicStats.map((t) => t.topic))),
  ).reduce(
    (acc, topic) => ({
      ...acc,
      [topic]: {
        label: topic,
        color: getTopicColor(topic),
      },
    }),
    {},
  );

  const chartData = data[currentTimeIndex].topicStats.map((stat) => ({
    ...stat,
    fill: `var(--color-${stat.topic})`,
  }));

  console.log(chartConfig);
  console.log(chartData);
  return (
    <div>
      <TrendingTopicsControls
        interval={interval}
        limit={limit}
        exclude={exclude}
        onIntervalChange={(value) => {
          setInterval(value);
          setCurrentTimeIndex(0); // Reset animation index
        }}
        onLimitChange={(value) => {
          setLimit(value);
          setCurrentTimeIndex(0);
        }}
        onExcludeChange={(value) => {
          setExclude(value);
          setCurrentTimeIndex(0);
        }}
      />

      <div className="mt-8">
        <ChartContainer config={chartConfig} className="h-[500px]">
          <BarChart
            layout="vertical"
            data={chartData}
            margin={{ top: 20, right: 30, left: 120, bottom: 5 }}
          >
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="eventCount" type="number" />
            <YAxis
              type="category"
              dataKey="topic"
              width={100}
              tick={{ fontSize: 12 }}
              interval={0}
            />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Bar
              dataKey="eventCount"
              radius={[0, 4, 4, 0]}
              maxBarSize={30}
              animationDuration={800}
            />
          </BarChart>
        </ChartContainer>

        <div className="mt-6 flex gap-2 overflow-x-auto pb-2">
          {data.map((timeData, index) => (
            <div
              key={timeData.time}
              className={`px-3 py-1 rounded text-sm whitespace-nowrap ${
                index === currentTimeIndex
                  ? "bg-primary text-primary-foreground"
                  : "bg-muted"
              }`}
            >
              {interval === "day"
                ? new Date(timeData.time).toLocaleDateString()
                : new Date(timeData.time).toLocaleTimeString()}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
