import {
  ConsumptionUtil,
  ConsumptionHelpers,
  sql,
  join_queries,
  Sql,
} from "@514labs/moose-lib";
import { ParsedLogs } from "../datamodels/logs";
interface QueryParams {
  limit: string;
  offset: string;
  search: string;
  sortDir?: string;
  sortCol?: string;
  source?: string;
  severity?: string;
}

function orderBySql(orderBy: string | undefined, desc: string | undefined) {
  if (!orderBy || !desc) return sql``;
  switch (desc) {
    case "DESC":
      return sql`ORDER BY ${ConsumptionHelpers.column(orderBy)} DESC`;
    case "ASC":
    default:
      return sql`ORDER BY ${ConsumptionHelpers.column(orderBy)} ASC`;
  }
}

export default async function handle(
  {
    limit = "10",
    offset = "0",
    sortDir = "DESC",
    sortCol,
    source,
    search,
    severity,
  }: QueryParams,
  { client }: ConsumptionUtil,
) {
  const logSql = createFilterLogSql({
    sortDir,
    sortCol,
    source,
    search,
    severity,
  });

  const response = client.query(
    sql`${logSql} LIMIT ${parseInt(limit)} OFFSET ${parseInt(offset)}`,
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

interface LogQuery {
  search: string;
  sortDir?: string;
  sortCol?: string;
  source?: string;
  severity?: string;
}

export function createFilterLogSql({
  sortDir = "DESC",
  sortCol,
  source,
  search,
  severity,
}: LogQuery) {
  const sort = orderBySql(sortCol, sortDir);

  const values: Sql[] = [];
  if (search) {
    values.push(sql`length(multiMatchAllIndices(message, patterns)) > 0`);
  }
  if (source) {
    values.push(sql`source LIKE ${`%${source}%`}`);
  }

  if (severity) {
    values.push(sql`has(splitByString(',', ${severity}), severityLevel)`);
  }

  const whereFilter =
    values.length > 0
      ? join_queries({
          prefix: "WHERE ",
          values: values,
          separator: " AND ",
        })
      : sql``;

  const pattern = `(?i)${search}`;

  const searchPattern = search ? sql`WITH [${pattern}] as patterns` : sql``;
  return sql`${searchPattern} SELECT *, COUNT(*) OVER() AS totalRowCount FROM ParsedLogs_0_6 ${whereFilter} ${sort}`;
}
