import { TimeUnit, timeUnitToSeconds } from "@/lib/time-utils";
import { DateRange, rangeToNum } from "./time-query";

interface QueryParams {
  dateRange: DateRange;
  interval: TimeUnit;
  breakdown: string[];
}

export function getChartData(props: QueryParams) {
  const url = new URL("http://localhost:4000/consumption/event_timeseries");

  const step = timeUnitToSeconds[props.interval];

  const to = Math.floor(Date.now() / 1000);
  const from =
    Math.floor(Date.now() / 1000) - rangeToNum[props.dateRange] * 3600;

  // Add step, from, and to parameters
  url.searchParams.append("step", step.toString());
  url.searchParams.append("from", from.toString());
  url.searchParams.append("to", to.toString());

  // Add breakdown parameters
  props.breakdown.forEach((item) => {
    if (item != null) {
      url.searchParams.append("breakdown", item);
    }
  });

  const test = fetch(url.toString())
    .then((response) => response.json())
    .then((data) => data)
    .catch((error) => console.error("Error:", error));

  return test;
}

interface TableQueryParams {
  dateRange: DateRange;
  breakdown: string[];
}

export function getTableData(props: TableQueryParams) {
  const url = new URL("http://localhost:4000/consumption/event_table");

  const to = Math.floor(Date.now() / 1000);
  const from =
    Math.floor(Date.now() / 1000) - rangeToNum[props.dateRange] * 3600;

  // Add step, from, and to parameters
  url.searchParams.append("from", from.toString());
  url.searchParams.append("to", to.toString());

  // Add breakdown parameters
  props.breakdown.forEach((item) => {
    if (item != null) {
      url.searchParams.append("breakdown", item);
    }
  });

  const test = fetch(url.toString())
    .then((response) => response.json())
    .then((data) => data)
    .catch((error) => console.error("Error:", error));

  return test;
}
