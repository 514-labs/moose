"use client";

import { useState, useEffect } from "react";
import { CartesianGrid, XAxis, YAxis, BarChart, Bar } from "recharts";
import {
  Loader2,
  Rewind,
  Play,
  Pause,
  FastForward,
  ChevronRight,
  ChevronLeft,
} from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import mooseClient from "@/lib/moose-client";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart";
import { TrendingTopicsControls } from "./trending-topics-controls";
import { Button } from "@/components/ui/button";

export function TrendingTopicsChart() {
  const [currentTimeIndex, setCurrentTimeIndex] = useState(0);
  const [isPlaying, setIsPlaying] = useState(true);
  const [interval, setInterval] = useState("hour");
  const [limit, setLimit] = useState(10);
  const [exclude, setExclude] = useState("");

  const { data, isLoading, error } = useQuery({
    queryKey: ["topicTimeseries", interval, limit, exclude],
    queryFn: async () => {
      console.log("queryFn", interval, limit, exclude);
      const result = await mooseClient.consumptionTopicTimeseriesGet({
        interval,
        limit,
        exclude: exclude || undefined,
      });
      return result;
    },
  });

  // Updated animation logic
  useEffect(() => {
    if (!data || !isPlaying) return;

    const intervalId = window.setInterval(() => {
      setCurrentTimeIndex((prev) => (prev + 1) % data.length);
    }, 2000);

    return () => window.clearInterval(intervalId);
  }, [data, isPlaying]);

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
    (acc, topic, index) => ({
      ...acc,
      [topic]: {
        label: topic,
        color: `var(--chart-${(index % 20) + 1})`,
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
        <ChartContainer
          config={chartConfig}
          className="h-[500px] justify-center"
        >
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

        <div className="mt-6">
          <div className="flex items-center gap-4 mb-4 justify-center">
            <Button onClick={() => setCurrentTimeIndex(0)} variant="outline">
              <Rewind className="w-4 h-4" />
            </Button>
            <Button
              variant="outline"
              onClick={() =>
                setCurrentTimeIndex(
                  currentTimeIndex === 0
                    ? data.length - 1
                    : currentTimeIndex - 1,
                )
              }
            >
              <ChevronLeft className="w-4 h-4" />
            </Button>
            <Button onClick={() => setIsPlaying(!isPlaying)} variant="outline">
              {isPlaying ? (
                <Pause className="w-4 h-4" />
              ) : (
                <Play className="w-4 h-4" />
              )}
            </Button>
            <Button
              variant="outline"
              onClick={() => {
                setCurrentTimeIndex(
                  currentTimeIndex === data.length - 1
                    ? 0
                    : currentTimeIndex + 1,
                );
              }}
            >
              <ChevronRight className="w-4 h-4" />
            </Button>
            <Button
              onClick={() => setCurrentTimeIndex(data.length - 1)}
              variant="outline"
            >
              <FastForward className="w-4 h-4" />
            </Button>
          </div>

          <div className="flex gap-2 overflow-x-auto pb-2 justify-center">
            {data.map((timeData, index) => (
              <Button
                key={timeData.time}
                onClick={() => {
                  setCurrentTimeIndex(index);
                  setIsPlaying(false);
                }}
                className={`px-3 py-1 rounded text-sm whitespace-nowrap cursor-pointer hover:bg-secondary/80 hover:text-secondary-foreground transition-colors ${
                  index === currentTimeIndex
                    ? "bg-secondary text-secondary-foreground font-medium"
                    : "bg-muted text-muted-foreground"
                }`}
              >
                {interval === "day"
                  ? new Date(timeData.time).toLocaleDateString()
                  : new Date(timeData.time).toLocaleTimeString()}
              </Button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
