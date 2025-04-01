export * from "./datamodels/models";

export * from "./functions/process";
export * from "./apis/bar";

import typia from "typia";
import { MaterializedView } from "@514labs/moose-lib/dist/dmv2";

interface BarAggregated {
  dayOfMonth: number & typia.tags.Type<"int64">;
  totalRows: number & typia.tags.Type<"int64">;
  rowsWithText: number & typia.tags.Type<"int64">;
  totalTextLength: number & typia.tags.Type<"int64">;
  maxTextLength: number & typia.tags.Type<"int64">;
}

export const BarAggregatedMV = new MaterializedView<BarAggregated>({
  tableName: "BarAggregated",
  materializedViewName: "BarAggregated_MV",
  orderByFields: ["dayOfMonth"],
  selectStatement: `SELECT
    toDayOfMonth(utcTimestamp) as dayOfMonth,
    count(primaryKey) as totalRows,
    countIf(hasText) as rowsWithText,
    sum(textLength) as totalTextLength,
    max(textLength) as maxTextLength
  FROM Bar
  GROUP BY toDayOfMonth(utcTimestamp)
  `,
});
