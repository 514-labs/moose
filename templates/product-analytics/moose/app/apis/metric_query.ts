import {
  ConsumptionUtil,
  ConsumptionHelpers,
  join_queries,
  sql,
} from "@514labs/moose-lib";
import { QueryFormData, createFilter, decodeQuery } from "../helpers/types";

export interface QueryParams {
  query: string;
  from: string;
  to: string;
  orderBy: string;
  desc: string;
}

function buildSelect(form: QueryFormData) {
  if (form.grouping.length === 0) {
    return sql`*`;
  }

  const groupingCols = form.grouping.map(
    (group) => sql`${ConsumptionHelpers.column(group.property)}`,
  );

  return join_queries({
    values: groupingCols,
    suffix: ", count(*) as count",
    separator: ", ",
  });
}

function orderBySql(orderBy: string | undefined, desc: string | undefined) {
  if (!orderBy || !desc) return sql``;
  switch (desc) {
    case "true":
      return sql`ORDER BY ${ConsumptionHelpers.column(orderBy)} DESC`;
    case "false":
    default:
      return sql`ORDER BY ${ConsumptionHelpers.column(orderBy)} ASC`;
  }
}

export default async function handle(
  { query, from, to, orderBy, desc }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  const queryForm = decodeQuery(query);

  if (!queryForm.metricName) throw new Error("Metric name is required");
  const timeFilters = [
    { property: "timestamp", operator: ">=", value: parseInt(from) },
    { property: "timestamp", operator: "<", value: parseInt(to) },
  ];

  const filterSql = [...timeFilters, ...queryForm.filter].map(createFilter);
  const filterQuery =
    filterSql.length > 0
      ? join_queries({
          prefix: "WHERE ",
          values: filterSql,
          separator: " AND ",
        })
      : sql``;

  const selectQuery = buildSelect(queryForm);

  const groupingSql = queryForm.grouping.map((group) =>
    ConsumptionHelpers.column(group.property),
  );

  const groupingQuery =
    groupingSql.length > 0
      ? join_queries({ prefix: "GROUP BY", values: groupingSql })
      : sql``;

  return client.query(
    sql`SELECT
    ${selectQuery}
    FROM
    ${ConsumptionHelpers.table(queryForm.metricName)}
    ${filterQuery}
    ${groupingQuery}
    ${orderBySql(orderBy, desc)}
    LIMIT 100`,
  );
}
