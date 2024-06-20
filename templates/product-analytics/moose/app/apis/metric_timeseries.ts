import {
  ConsumptionUtil,
  ConsumptionHelpers,
  join_queries,
  sql,
} from "@514labs/moose-lib";
import {
  QueryFormData,
  buildGrouping,
  createFilter,
  decodeQuery,
} from "../helpers/types";
import { getDefaultFrom, getDefaultTo } from "../helpers/utils";
export interface QueryParams {
  query: string;
  step: string;
  from: string;
  to: string;
}

export default async function handle(
  {
    query,
    from = getDefaultFrom(),
    to = getDefaultTo(),
    step = "3600",
  }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  const queryForm = decodeQuery(query);

  if (!queryForm.metricName) throw new Error("Metric name is required");

  const filters = [
    { property: "timestamp", operator: ">=", value: parseInt(from) },
    { property: "timestamp", operator: "<", value: parseInt(to) },
  ];

  const filterSql = [...filters, ...queryForm.filter].map(createFilter);
  const filterQuery =
    filterSql.length > 0
      ? join_queries({
          prefix: "WHERE ",
          values: filterSql,
          separator: " AND ",
        })
      : sql``;

  const groupList = queryForm.grouping.map(
    (group) => sql`${ConsumptionHelpers.column(group.property)}`,
  );

  const groupListSql =
    groupList.length > 0
      ? join_queries({ values: groupList, separator: ", ", suffix: "," })
      : sql``;

  const grouping = buildGrouping(queryForm.grouping);

  const sqlQuery = sql`SELECT 
  ${groupListSql}
  groupArray((timestamp, count)) as timeseries,
  sum(count) as total_count
FROM (
  SELECT 
      ${groupListSql}
      toStartOfInterval(timestamp, interval 3600 second) as timestamp,
      count(*) as count
  FROM ${ConsumptionHelpers.table(queryForm.metricName)}
  ${filterQuery}
  GROUP BY ${groupListSql} timestamp
  ORDER BY ${groupListSql} timestamp ASC
  WITH FILL FROM toStartOfInterval(fromUnixTimestamp(${parseInt(from)}), interval ${parseInt(step)} second) TO fromUnixTimestamp(${parseInt(to)}) STEP ${parseInt(step)}
)
${grouping}
ORDER BY total_count DESC`;

  return client.query(sqlQuery);
}
