import { TimeUnit, timeUnitToSeconds } from "@/lib/time-utils";

export enum DateRange {
  "1H" = "1H",
  "Today" = "1D",
  "3D" = "3D",
  "7D" = "7D",
  "30D" = "30D",
}
export const rangeToNum = {
  [DateRange["1H"]]: 1,
  [DateRange.Today]: 1 * 24,
  [DateRange["3D"]]: 1 * 24 * 3,
  [DateRange["7D"]]: 1 * 24 * 7,
  [DateRange["30D"]]: 1 * 24 * 30,
};

export function timeToClickHouseInterval(timeUnit: TimeUnit) {
  switch (timeUnit) {
    case TimeUnit.DAY:
      return "toStartOfDay";
    case TimeUnit.HOUR:
      return "toStartOfHour";
    case TimeUnit.MINUTE:
      return "toStartOfMinute";
  }
}

export function createDateStub(range: DateRange) {
  return `WHERE timestamp >= toDateTime(now() - interval ${rangeToNum[range]} hour)`;
}

export const timeseries = (dateRange: DateRange, timeUnit: TimeUnit) => {
  const start = `${timeToClickHouseInterval(timeUnit)}(timestampAdd(now(), interval -${rangeToNum[dateRange]} hour)) as start`;
  const end = `${timeToClickHouseInterval(timeUnit)}(now()) as end`;
  return `
    with ${start}, ${end}
      select
      arrayJoin(
          arrayMap(
              x -> toDateTime(x),
              range(toUInt32(start), toUInt32(timestampAdd(end, interval 1 hour)), ${timeUnitToSeconds[timeUnit]})
          )
      ) as timestamp
      where timestamp <= now()`;
};
