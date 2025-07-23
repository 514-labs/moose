import typia from "typia";
import { RefreshableMaterializedView, duration, sql } from "@514labs/moose-lib";
import { BarPipeline } from "../ingest/models";

interface BarRefreshableStats {
  dayOfWeek: number & typia.tags.Type<"int64">;
  totalRows: number & typia.tags.Type<"int64">;
  avgTextLength: number;
  maxTextLength: number & typia.tags.Type<"int64">;
}

const barTable = BarPipeline.table!;
const barColumns = barTable.columns;

// Example: Refreshable materialized view that recalculates weekly stats every hour
export const BarRefreshableStatsMV =
  new RefreshableMaterializedView<BarRefreshableStats>({
    tableName: "BarRefreshableStats_NEW",
    materializedViewName: "BarRefreshableStats_RMV_NE",
    refreshEvery: duration.hours(1), // Refresh every hour
    orderByFields: ["dayOfWeek"],
    selectStatement: sql`SELECT
    toDayOfWeek(${barColumns.utcTimestamp}) as dayOfWeek,
    count(${barColumns.primaryKey}) as totalRows,
    avg(${barColumns.textLength}) as avgTextLength,
    max(${barColumns.textLength}) as maxTextLength
  FROM Bar
  GROUP BY toDayOfWeek(utcTimestamp)
  `,
    selectTables: [barTable],
  });
