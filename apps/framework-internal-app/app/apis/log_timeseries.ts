import { ConsumptionUtil } from "@514labs/moose-lib";
import { createFilterLogSql } from "./log_query";

interface QueryParams {
  search: string;
  sortDir?: string;
  sortCol?: string;
  source?: string;
  severity?: string;
}

export default async function handle(
  { sortDir = "DESC", sortCol, source, search, severity }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  const logSql = createFilterLogSql({
    sortDir,
    sortCol,
    source,
    search,
    severity,
  });

  return client.query(
    sql`
    WITH filtered_logs as (
      ${logSql}
    )
    SELECT 
    toStartOfInterval(date, interval (3600 * 6) second) as date,
        sum(severityLevel = 'INFO') as info,
    sum(severityLevel = 'ERROR') as error,
    sum(severityLevel = 'WARN') as warn,
    sum(severityLevel = 'DEBUG') as debug
FROM filtered_logs
GROUP BY date
ORDER BY date
WITH FILL FROM toStartOfInterval(parseDateTimeBestEffort('2024-06-27 04:21:32'), interval 3600 second)
TO parseDateTimeBestEffort('2024-07-08 07:12:13') STEP interval (3600 * 6) second`,
  );
}
