import typia from "typia";
import { MaterializedView } from "@514labs/moose-lib";
import { BarPipeline } from "../ingest/models";

interface BarAggregated {
  dayOfMonth: number & typia.tags.Type<"int64">;
  totalRows: number & typia.tags.Type<"int64">;
  rowsWithText: number & typia.tags.Type<"int64">;
  totalTextLength: number & typia.tags.Type<"int64">;
  maxTextLength: number & typia.tags.Type<"int64">;
}

const BarTable = BarPipeline.table!;

const query = `SELECT
    toDayOfMonth(${BarTable.columns.utcTimestamp.name}) as dayOfMonth,
    count(${BarTable.columns.primaryKey.name}) as totalRows,
    countIf(${BarTable.columns.hasText.name}) as rowsWithText,
    sum(${BarTable.columns.textLength.name}) as totalTextLength,
    max(${BarTable.columns.textLength.name}) as maxTextLength
  FROM ${BarTable.name}
  GROUP BY toDayOfMonth(${BarTable.columns.utcTimestamp.name})
  `;

export const BarAggregatedMV = new MaterializedView<BarAggregated>({
  tableName: "BarAggregated",
  materializedViewName: "BarAggregated_MV",
  orderByFields: ["dayOfMonth"],
  selectStatement: query,
});
