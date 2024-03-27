import { analyticsQuery } from "./analytics_mv";
import { analyticsSessions } from "./analytics_sessions";
import { eventTables } from "../config";
import { DateRange, timeseries } from "./time-query";
import { createCTE } from "./util";
import { TimeUnit } from "@/lib/time-utils";
import { getData } from "@/app/data";

export const hits = (sessionTable: string, range: DateRange) => {
  return `
    select
            date as timestamp,
            session_id,
            uniq(session_id) as visits,
            countMerge(hits) as pageviews,
            case when min(first_hit) = max(latest_hit) then 1 else 0 end as is_bounce,
            max(latest_hit) as latest_hit_aux,
            min(first_hit) as first_hit_aux
        from ${sessionTable}
        group by timestamp, session_id

    `;
};

export const hitData = (tableName: string) => {
  return `
    select
    timestamp,
    uniq(session_id) as visits,
    sum(pageviews) as pageviews,
    100 * sum(case when latest_hit_aux = first_hit_aux then 1 end) / visits as bounce_rate,
    avg(latest_hit_aux - first_hit_aux) as avg_session_sec
from ${tableName}
group by timestamp`;
};

export const kpiTimeseries = (
  timeseriesTable: string,
  hitDataTable: string
) => {
  return `
    select a.timestamp, b.visits, b.pageviews, b.bounce_rate, b.avg_session_sec
    from ${timeseriesTable} a
    left join ${hitDataTable} b using timestamp`;
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
    [KPICtes.timeseries]: timeseries(range, TimeUnit.HOUR),
    [KPICtes.analytics]: analyticsQuery(eventTables[0].tableName),
    [KPICtes.sessions]: analyticsSessions(KPICtes.analytics),
    [KPICtes.hits]: hits(KPICtes.sessions, range),
    [KPICtes.data]: hitData(KPICtes.hits),
  };

  return (
    createCTE(ctes) +
    `select a.timestamp, b.visits, b.pageviews, b.bounce_rate, b.avg_session_sec
    from ${KPICtes.timeseries} a
    left join ${KPICtes.data} b using timestamp`
  );
};

export interface KPI {
  timestamp: string;
  visits: number;
  pageviews: number;
  bounce_rate: number;
  avg_session_sec: number;
}

export function getKPI(range: DateRange) {
  return getData(kpiQuery(range)) as Promise<KPI[]>;
}
