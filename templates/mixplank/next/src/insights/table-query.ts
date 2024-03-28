import { TimeUnit } from "@/lib/time-utils";
import { EventTable } from "../app/events";
import {
  DateRange,
  createDateStub,
  timeToClickHouseInterval,
  timeseries,
} from "./time-query";
import { createCTE } from "./util";
import { getData } from "@/app/data";
import { sessionData } from "./sessions";

function createColumnSnippet(columnNames: string[]) {
  if (columnNames.length == 0) {
    return "";
  }
  return ", " + columnNames.map((b) => `${b}`).join(", ");
}
export const getUniqueValues = (tableName: string, columnNames: string[]) => {
  const columnNamesString =
    columnNames.length != 0 ? "distinct " + columnNames.join(", ") : "*";
  return `select ${columnNamesString} from ${tableName}`;
};

function allCombinations(
  tableOne: string,
  tableTwo: string,
  columnNames: string[]
) {
  return `select timestamp${createColumnSnippet(columnNames)} from ${tableOne}, ${tableTwo}`;
}

export function tableQuery(
  events: EventTable[],
  dateRange: DateRange,
  columnNames: string[]
) {
  return events
    .map(
      (event) =>
        `SELECT  '${event.eventName}' AS event_name, timestamp, session_id ${createColumnSnippet(columnNames)} FROM ${event.tableName} ${createDateStub(dateRange)}`
    )
    .join("\nUNION DISTINCT\n");
}

export function virtualTableQuery(
  events: EventTable[],
  dateRange: DateRange,
  columnNames: string[]
) {
  const cte = {
    ["virtual_session"]: sessionData(),
    ["combined_events"]: tableQuery(events, dateRange, columnNames),
  };
  return (
    createCTE(cte) +
    `SELECT timestamp, event_name, session_id ${createColumnSnippet(columnNames)} from combined_events
ORDER BY timestamp DESC`
  );
}

export function data(
  tableName: string,
  columnNames: string[],
  interval: TimeUnit
) {
  return `
    SELECT
        ${timeToClickHouseInterval(interval)}(timestamp) AS timestamp,
        count() AS count
        ${createColumnSnippet(columnNames)}
    FROM
        ${tableName}
    GROUP BY
        timestamp
        ${createColumnSnippet(columnNames)}
`;
}

enum Ctes {
  data = "data",
  timeseries = "timeseries",
  uniqValues = "uniqValues",
  allTables = "allTables",
  combinations = "combinations",
  analytics = "analytics",
  sessions = "sessions",
}

export function chartQuery(
  events: EventTable[],
  dateRange: DateRange,
  interval: TimeUnit,
  columnNames: string[]
) {
  const ctes = {
    [Ctes.timeseries]: timeseries(dateRange, interval),
    [Ctes.allTables]: virtualTableQuery(events, dateRange, columnNames),
    [Ctes.uniqValues]: getUniqueValues(Ctes.allTables, columnNames),
    [Ctes.combinations]: allCombinations(
      Ctes.timeseries,
      Ctes.uniqValues,
      columnNames
    ),
    ["data"]: data(Ctes.allTables, columnNames, interval),
  };

  const join =
    columnNames.length != 0
      ? " AND " +
        columnNames
          .map((b) => `${Ctes.combinations}.${b} = data.${b}`)
          .join(" AND ")
      : "";

  const combinations =
    columnNames.length != 0
      ? ", " + columnNames.map((b) => `${Ctes.combinations}.${b}`).join(", ")
      : "";

  return (
    createCTE(ctes) +
    `SELECT
    ${Ctes.combinations}.timestamp AS timestamp
    ${combinations},
    if(data.count IS NULL, 0, data.count) AS count
FROM
    ${Ctes.combinations}
LEFT JOIN
    data ON ${Ctes.combinations}.timestamp = data.timestamp ${join}
ORDER BY
    timestamp`
  );
}

export function getChartQueryData(
  events: EventTable[],
  dateRange: DateRange,
  interval: TimeUnit,
  columnNames: string[]
) {
  if (events.length == 0) return Promise.resolve([]);
  return getData(
    chartQuery(events, dateRange, interval, columnNames)
  ) as Promise<object & { timestamp: string }[]>;
}

export function getTableQueryData(
  events: EventTable[],
  dateRange: DateRange,
  columnNames: string[]
) {
  if (events.length == 0) return Promise.resolve([]);
  return getData(virtualTableQuery(events, dateRange, columnNames));
}
