import { analyticsQuery } from "./analytics_mv";
import { analyticsSessions } from "./analytics_sessions";
import { eventTables } from "./event-tables";
import { DateRange, rangeToNum } from "./time-query";
import { createCTE } from "./util";

// generate time series to join against
export const timeseries = (dateRange: DateRange) => {
  const start = `toStartOfDay(timestampAdd(today(), interval -${rangeToNum[dateRange]} day)) as start`;
  const end = `toStartOfDay(today()) as end`;
  return `
  with ${start}, ${end}
    select
    arrayJoin(
        arrayMap(
            x -> toDateTime(x),
            range(toUInt32(start),toUInt32(timestampAdd(end, interval 1 day)), 3600) -- 3600 seconds = 1 hour
        )
    ) as date
    where date <= now()`;
};

export const hits = (sessionTable: string, range: DateRange) => {
  return `
    select
            date,
            session_id,
            uniq(session_id) as visits,
            countMerge(hits) as pageviews,
            case when min(first_hit) = max(latest_hit) then 1 else 0 end as is_bounce,
            max(latest_hit) as latest_hit_aux,
            min(first_hit) as first_hit_aux
        from ${sessionTable}
        where date >= timestampAdd(today(), interval -${rangeToNum[range]} day)
        and date <= today()
        group by date, session_id

    `;
};

export const hitData = (tableName: string) => {
  return `
    select
    date,
    uniq(session_id) as visits,
    sum(pageviews) as pageviews,
    100 * sum(case when latest_hit_aux = first_hit_aux then 1 end) / visits as bounce_rate,
    avg(latest_hit_aux - first_hit_aux) as avg_session_sec
from ${tableName}
group by date`;
};

export const kpiTimeseries = (
  timeseriesTable: string,
  hitDataTable: string
) => {
  return `
    select a.date, b.visits, b.pageviews, b.bounce_rate, b.avg_session_sec
    from ${timeseriesTable} a
    left join ${hitDataTable} b using date`;
};

enum KPICtes {
  analytics = "analytics",
  timeseries = "timeseries",
  sessions = "sessions",
  hits = "hits",
  data = "data",
}

export const kpiQuery = (range: DateRange) => {
  const ctes = {
    [KPICtes.timeseries]: timeseries(range),
    [KPICtes.analytics]: analyticsQuery(eventTables),
    [KPICtes.sessions]: analyticsSessions(KPICtes.analytics),
    [KPICtes.hits]: hits(KPICtes.sessions, range),
    [KPICtes.data]: hitData(KPICtes.hits),
  };

  return (
    createCTE(ctes) +
    `select a.date, b.visits, b.pageviews, b.bounce_rate, b.avg_session_sec
    from ${KPICtes.timeseries} a
    left join ${KPICtes.data} b using date`
  );
};
