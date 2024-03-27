import { TimeUnit, timeUnitToSeconds } from "@/lib/time-utils";

export enum DateRange {
  "Today" = "1D",
  "3D" = "3D",
  "7D" = "7D",
  "30D" = "30D",
}
export const rangeToNum = {
  [DateRange.Today]: 1,
  [DateRange["3D"]]: 3,
  [DateRange["7D"]]: 7,
  [DateRange["30D"]]: 30,
};
export function createDateStub(range: DateRange) {
  return `WHERE timestamp >= toDate(today() - ${rangeToNum[range]})`;
}

export const timeseries = (dateRange: DateRange, timeUnit: TimeUnit) => {
  const start = `toStartOfDay(timestampAdd(today(), interval -${rangeToNum[dateRange]} day)) as start`;
  const end = `toStartOfDay(today()) as end`;
  return `
    with ${start}, ${end}
      select
      arrayJoin(
          arrayMap(
              x -> toDateTime(x),
              range(toUInt32(start), toUInt32(timestampAdd(end, interval 1 day)), ${timeUnitToSeconds[timeUnit]})
          )
      ) as timestamp
      where timestamp <= now()`;
};
