import { pageViewEvent } from "@/app/events";
import { createCTE } from "./util";

// These queries create virtual session events
function sessions() {
  return `SELECT
          toStartOfHour(timestamp) AS date,
          session_id,
          minSimpleState(timestamp) AS first_hit,
          maxSimpleState(timestamp) AS latest_hit,
          countState() AS hits
      FROM ${pageViewEvent.tableName}
      GROUP BY date, session_id`;
}

export function sessionsQuery() {
  return `SELECT
    timestamp,
      session_id,
      event_name,
    FROM (
      SELECT
        session_id,
        'session_started' AS event_name,
        minSimpleState(timestamp) AS timestamp
      FROM ${pageViewEvent.tableName}
      GROUP BY session_id

      UNION ALL

      SELECT
        session_id,
        'session_ended' AS event_name,
        maxSimpleState(timestamp) AS timestamp
      FROM ${pageViewEvent.tableName}
      GROUP BY session_id
    )
    ORDER BY session_id, event_name`;
}

export const hits = (sessionTable: string) => {
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
      session_id,
      uniq(session_id) as visits,
      sum(pageviews) as pageviews,
      100 * sum(case when latest_hit_aux = first_hit_aux then 1 end) / visits as bounce_rate,
      avg(latest_hit_aux - first_hit_aux) as avg_session_sec
  from ${tableName}
  group by session_id, timestamp`;
};

export const sessionData = () => {
  const ctes = {
    ["sessionEvents"]: sessionsQuery(),
    ["sessions"]: sessions(),
    ["hits"]: hits("sessions"),
    ["data"]: hitData("hits"),
  };

  return (
    createCTE(ctes) +
    `SELECT
  s.timestamp,
  s.session_id,
  s.event_name,
  d.avg_session_sec,
  d.visits,
  d.pageviews,
FROM sessionEvents AS s
JOIN data AS d ON s.session_id = d.session_id
ORDER BY s.timestamp, s.session_id, s.event_name`
  );
};
