import { EventTable, eventTables } from "./event-tables";
import { DateRange, createDateStub, rangeToNum } from "./time-query";
import { createCTE } from "./util";

export const timeseries = (dateRange: DateRange) => {
  const start = `toStartOfDay(timestampAdd(today(), interval -${rangeToNum[dateRange]} day)) as start`;
  const end = `toStartOfDay(today()) as end`;
  return `
    with ${start}, ${end}
      select
      arrayJoin(
          arrayMap(
              x -> toDateTime(x),
              range(toUInt32(start), toUInt32(timestampAdd(end, interval 1 day)), 3600) -- 3600 seconds = 1 hour
          )
      ) as date
      where date <= now()`;
};

export function tableQuery(events: EventTable[], dateRange: DateRange) {
  const all_tables = events
    .map(
      (event) =>
        `SELECT  '${event.eventName}' AS event_name, timestamp, session_id, location, pathname FROM ${event.tableName} ${createDateStub(dateRange)}`
    )
    .join("\nUNION ALL\n");
  return `WITH CombinedEvents AS (
            ${all_tables}
        ) FROM CombinedEvents
        SELECT session_id, timestamp, event_name, pathname, location
    ORDER BY timestamp DESC`;
}

export function data(tableName: string) {
  return `
  SELECT
  hour,
  arrayJoin(paths) AS pathname,
  arrayJoin(counts) AS count
FROM (
  SELECT
      toStartOfHour(event_time) AS hour,
      groupArray(pathname) AS paths,
      groupArray(count) AS counts
  FROM (
      SELECT
          toStartOfHour(timestamp) AS event_time,
          pathname,
          count() AS count
      FROM
          ${tableName}
      GROUP BY
          event_time,
          pathname
      ORDER BY
          event_time,
          pathname
  )
  GROUP BY
      hour
)
ORDER BY
  hour,
  pathname
  `;
}

enum Ctes {
  data = "data",
  timeseries = "timeseries",
}
/*
export function timeSeriesData(dateRange: DateRange) {
  const ctes = {
    [Ctes.timeseries]: timeseries(dateRange),
    [Ctes.data]: tableQuery(eventTables, dateRange),
  };
  const query =
    createCTE(ctes) +
    `select a.date, b.event_name, b.session_id, b.pathname, b.timestamp
  from ${Ctes.timeseries} a
  left join ${Ctes.data} b ON toStartOfHour(a.date) = toStartOfHour(b.timestamp)`;
  return query;
}

*/
export function testFunc(dateRange: DateRange) {
  const ctes = {
    [Ctes.timeseries]: timeseries(dateRange),
    ["yo"]: tableQuery(eventTables, dateRange),
    ["data"]: data("yo"),
  };

  return createCTE(ctes) + `select * from data`;
}
