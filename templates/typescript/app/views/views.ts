import typia from "typia";
import { MaterializedView, sql } from "@514labs/moose-lib";
import { BarPipeline } from "../ingest/models";

interface BarAggregated {
  dayOfMonth: number & typia.tags.Type<"int64">;
  totalRows: number & typia.tags.Type<"int64">;
  rowsWithText: number & typia.tags.Type<"int64">;
  totalTextLength: number & typia.tags.Type<"int64">;
  maxTextLength: number & typia.tags.Type<"int64">;
}

const BTCols = BarPipeline.table!.columns;

const query = `SELECT
    toDayOfMonth(${BTCols.utcTimestamp.name}) as dayOfMonth,
    count(${BTCols.primaryKey.name}) as totalRows,
    countIf(${BTCols.hasText.name}) as rowsWithText,
    sum(${BTCols.textLength.name}) as totalTextLength,
    max(${BTCols.textLength.name}) as maxTextLength
  FROM ${BarPipeline.table!.name}
  GROUP BY toDayOfMonth(${BTCols.utcTimestamp.name})
  `;

export const BarAggregatedMV = new MaterializedView<BarAggregated>({
  tableName: "BarAggregated",
  materializedViewName: "BarAggregated_MV",
  orderByFields: ["dayOfMonth"],
  selectStatement: query,
});
