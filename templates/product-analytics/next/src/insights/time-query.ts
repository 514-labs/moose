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
