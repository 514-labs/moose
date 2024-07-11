import {
  ConsumptionUtil,
  ConsumptionHelpers,
  join_queries,
} from "@514labs/moose-lib";
import { createFilter } from "../helpers/types";

export interface QueryParams {
  metricName: string;
  propertyName: string;
  filters: string;
}

export default async function handle(
  { metricName, propertyName, filters }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  const filter = JSON.parse(decodeURIComponent(filters)) as {
    property: string;
    value: string;
    operator: string;
  }[];

  const filterSql = filter.map(createFilter);

  const filterQuery =
    filterSql.length > 0
      ? join_queries({
          prefix: "WHERE ",
          values: filterSql,
          separator: " AND ",
        })
      : sql``;

  return client.query(
    sql`SELECT ${ConsumptionHelpers.column(propertyName)} as value,
    count(*) as count
    FROM ${ConsumptionHelpers.table(metricName)}
    ${filterQuery}
    GROUP BY ${ConsumptionHelpers.column(propertyName)}
    ORDER BY count DESC LIMIT 50`,
  );
}
