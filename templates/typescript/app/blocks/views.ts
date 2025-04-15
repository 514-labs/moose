import typia from "typia";
import { MaterializedView, sql } from "@514labs/moose-lib";
import { BarPipeline } from "../datamodels/models";

interface BarAggregated {
  dayOfMonth: number & typia.tags.Type<"int64">;
  totalRows: number & typia.tags.Type<"int64">;
  rowsWithText: number & typia.tags.Type<"int64">;
  totalTextLength: number & typia.tags.Type<"int64">;
  maxTextLength: number & typia.tags.Type<"int64">;
}

const barTable = BarPipeline.table!;
export const BarAggregatedMV = new MaterializedView<BarAggregated>({
  tableName: "BarAggregated",
  materializedViewName: "BarAggregated_MV",
  orderByFields: ["dayOfMonth"],
  selectStatement: sql`SELECT
    toDayOfMonth(${barTable.columns.utcTimestamp}) as dayOfMonth,
    count(${barTable.columns.primaryKey}) as totalRows,
    countIf(${barTable.columns.hasText}) as rowsWithText,
    sum(${barTable.columns.textLength}) as totalTextLength,
    max(${barTable.columns.textLength}) as maxTextLength
  FROM ${barTable}
  GROUP BY toDayOfMonth(utcTimestamp)
  `,
});
