import { EventTable } from "../config";
import { virtualTableQuery } from "./table-query";
import { DateRange, createDateStub } from "./time-query";
import { createCTE } from "./util";

type FunnelQuery = EventTable[];

function createColumnSnippet(columnNames: string[]) {
  if (columnNames.length == 0) {
    return "";
  }
  return ", " + columnNames.map((b) => `${b}`).join(", ");
}

function createPartitionSnippet(columnNames: string[]) {
  if (columnNames.length == 0) {
    return "";
  }
  return `PARTITION BY ${columnNames.join(", ")}`;
}

export function createCombinedEvents(
  events: EventTable[],
  dateRange: DateRange,
  columnNames: string[]
) {
  return events
    .map(
      (event) =>
        `SELECT session_id, timestamp, '${event.eventName}' AS event_name ${createColumnSnippet(columnNames)} FROM ${event.tableName} ${createDateStub(dateRange)}`
    )
    .join("\nUNION ALL\n");
}

function levelCounts(
  tableName: string,
  eventsTable: EventTable[],
  columnNames: string[]
) {
  return `SELECT session_id${createColumnSnippet(columnNames)},
  windowFunnel(60*60*24*30*24)(timestamp,
      ${windowFunnelEventsList(eventsTable)}
  ) AS level
FROM ${tableName}
GROUP BY ALL
HAVING level >= 1`;
}

function windowFunnelEventsList(events: EventTable[]) {
  return events.map((event) => `event_name = '${event.eventName}'`).join(",");
}

export const createFunnelQuery = (
  queryParams: FunnelQuery,
  dateRange: DateRange,
  columnNames: string[]
) => {
  const cleanEvents = queryParams.filter((ev) => ev != null);
  if (!cleanEvents || cleanEvents.length == 0) {
    return "";
  }

  const cte = {
    ["CombinedEvents"]: virtualTableQuery(cleanEvents, dateRange, columnNames),
    ["LevelCounts"]: levelCounts("CombinedEvents", cleanEvents, columnNames),
  };
  return (
    createCTE(cte) +
    `SELECT level${createColumnSnippet(columnNames)}, sum(count()) OVER (${createPartitionSnippet(columnNames)} ORDER BY level DESC) AS count
        FROM LevelCounts
        GROUP BY level${createColumnSnippet(columnNames)}
        ORDER BY level ASC;`
  );
};
