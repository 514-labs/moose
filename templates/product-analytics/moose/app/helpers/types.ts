import { sql, ConsumptionHelpers, join_queries } from "@514labs/moose-lib";

export const typeToValidOperators = {
  ["UInt64"]: ["=", ">", "<", ">=", "<=", "!="],
  ["Int32"]: ["=", ">", "<", ">=", "<=", "!="],
  ["String"]: ["=", "!=", "LIKE", "ILIKE", "CONTAINS"],
  ["DateTime"]: ["=", ">", "<", ">=", "<=", "!="],
  ["Nullable(String)"]: ["=", ">", "<", ">=", "<=", "!=", "LIKE"],
  ["Nullable(Bool)"]: ["=", "!="],
  ["Bool"]: ["=", "!="],
  ["DateTime('UTC')"]: ["=", ">", "<", ">=", "<=", "!="],
};

export interface QueryFormData {
  metricName: string;
  filter: { property: string; value: string; operator: string }[];
  grouping: { property: string }[];
}
export const decodeQuery = (query: string): QueryFormData =>
  JSON.parse(decodeURIComponent(query));

export function createFilter({
  property,
  value,
  operator,
}: {
  property: string;
  value: string | number;
  operator: string;
}) {
  switch (operator) {
    case "CONTAINS":
      return sql`${ConsumptionHelpers.column(property)} LIKE ${`%${value}%`}`;
    case "=":
      return sql`${ConsumptionHelpers.column(property)} = ${value}`;
    case "LIKE":
      return sql`${ConsumptionHelpers.column(property)} ILIKE ${`%${value}%`}`;
    case ">=":
      return sql`${ConsumptionHelpers.column(property)} >= ${value}`;
    case ">":
      return sql`${ConsumptionHelpers.column(property)} > ${value}`;
    case "<":
      return sql`${ConsumptionHelpers.column(property)} < ${value}`;
    case "<=":
      return sql`${ConsumptionHelpers.column(property)} <= ${value}`;
    case "!=":
      return sql`${ConsumptionHelpers.column(property)} != ${value}`;
    default:
      return sql`${ConsumptionHelpers.column(property)} = ${value}`;
  }
}

export function buildSelect(form: QueryFormData) {
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

export function buildGrouping(grouping: QueryFormData["grouping"]) {
  const groupingSql = grouping.map((group) =>
    ConsumptionHelpers.column(group.property),
  );

  return groupingSql.length > 0
    ? join_queries({ prefix: "GROUP BY", values: groupingSql })
    : sql``;
}
