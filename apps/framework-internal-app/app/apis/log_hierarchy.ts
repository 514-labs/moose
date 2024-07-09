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
    splitByString('::', source) AS categories,
    sum(severityLevel = 'INFO') as info,
    sum(severityLevel = 'ERROR') as error,
    sum(severityLevel = 'WARN') as warn,
    sum(severityLevel = 'DEBUG') as debug
FROM filtered_logs
GROUP BY source`,
  );
}
