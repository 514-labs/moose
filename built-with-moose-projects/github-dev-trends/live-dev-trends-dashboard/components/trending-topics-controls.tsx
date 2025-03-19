"use client";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface TrendingTopicsControlsProps {
  interval: string;
  limit: number;
  exclude: string;
  onIntervalChange: (value: string) => void;
  onLimitChange: (value: number) => void;
  onExcludeChange: (value: string) => void;
}

export function TrendingTopicsControls({
  interval,
  limit,
  exclude,
  onIntervalChange,
  onLimitChange,
  onExcludeChange,
}: TrendingTopicsControlsProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 p-4 bg-gray-50 rounded-lg">
      <div>
        <Label htmlFor="interval">Time Interval</Label>
        <Select value={interval} onValueChange={onIntervalChange}>
          <SelectTrigger id="interval">
            <SelectValue placeholder="Select interval" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="minute">Minute</SelectItem>
            <SelectItem value="hour">Hour</SelectItem>
            <SelectItem value="day">Day</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div>
        <Label htmlFor="limit">Number of Topics</Label>
        <Select
          value={(limit ?? 10).toString()}
          onValueChange={(value) => onLimitChange(parseInt(value))}
        >
          <SelectTrigger id="limit">
            <SelectValue placeholder="Select limit" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="5">5</SelectItem>
            <SelectItem value="10">10</SelectItem>
            <SelectItem value="15">15</SelectItem>
            <SelectItem value="20">20</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div>
        <Label htmlFor="exclude">Exclude Topics (comma separated)</Label>
        <Input
          id="exclude"
          value={exclude}
          onChange={(e) => onExcludeChange(e.target.value)}
          placeholder="e.g. javascript,python"
        />
      </div>
    </div>
  );
}
