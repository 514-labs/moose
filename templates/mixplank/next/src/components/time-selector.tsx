"use client";
import { DateRange } from "@/insights/time-query";
import { Button } from "./ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { TimeUnit } from "@/lib/time-utils";

interface Props {
  selectedRange: DateRange;
  setDateRange: (range: DateRange) => void;
  interval?: TimeUnit;
  setInterval?: (interval: TimeUnit) => void;
}

function DateButton({
  selectedRange,
  setDateRange,
  range,
}: Props & { range: DateRange }) {
  return (
    <Button
      className={`rounded-xl ${selectedRange == range && "bg-accent"}`}
      variant={"ghost"}
      onClick={() => setDateRange(range)}
    >
      {range}
    </Button>
  );
}

interface IntervalProps {
  interval: TimeUnit;
  setInterval: (interval: TimeUnit) => void;
}
function IntervalSelector({ interval, setInterval }: IntervalProps) {
  return (
    <Select
      value={interval}
      onValueChange={(val: TimeUnit) => setInterval(val)}
    >
      <SelectTrigger className="rounded-xl capitalize w-20">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={TimeUnit.MINUTE}>Minute</SelectItem>
        <SelectItem value={TimeUnit.HOUR}>Hour</SelectItem>
        <SelectItem value={TimeUnit.DAY}>Day</SelectItem>
      </SelectContent>
    </Select>
  );
}
export default function TimeSelector(props: Props) {
  return (
    <div className="flex gap-2 mx-4">
      <div className="">
        <DateButton {...props} range={DateRange["Today"]} />
        <DateButton {...props} range={DateRange["3D"]} />
        <DateButton {...props} range={DateRange["7D"]} />
        <DateButton {...props} range={DateRange["30D"]} />
      </div>
      {props.setInterval && props.interval && (
        <IntervalSelector
          interval={props.interval}
          setInterval={props.setInterval}
        />
      )}
    </div>
  );
}
