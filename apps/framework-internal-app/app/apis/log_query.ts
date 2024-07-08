import { ConsumptionUtil, ConsumptionHelpers } from "@514labs/moose-lib";
import { ParsedLogs } from "../datamodels/logs";
interface QueryParams {
  limit: string;
  offset: string;
  search: string;
  sortDir?: string;
  sortCol?: string;
}

export default async function handle(
  { limit = "10", offset = "0", sortDir = "DESC", sortCol }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  const limitInt = parseInt(limit);
  const offsetInt = parseInt(offset);
  const sort = sortCol
    ? sql`ORDER BY ${ConsumptionHelpers.column(sortCol)} ${sortDir}`
    : sql``;

  const response = client.query(
    sql`SELECT *, COUNT(*) OVER() AS totalRowCount FROM ParsedLogs_0_5 LIMIT ${limitInt} OFFSET ${offsetInt} ${sort}`,
  );

  const data = (await (await response).json()) as ParsedLogs &
    { totalRowCount: number }[];
  return {
    data: data,
    meta: {
      totalRowCount: data?.[0]?.totalRowCount ?? 0,
    },
  };
}
