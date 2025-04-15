import { ConsumptionApi } from "@514labs/moose-lib";
import { BarAggregatedMV } from "../blocks/views";
import { tags } from "typia";

// This file is where you can define your APIs to consume your data
interface QueryParams {
  orderBy?: "totalRows" | "rowsWithText" | "maxTextLength" | "totalTextLength";
  limit?: number;
  startDay?: number & tags.Type<"int32">;
  endDay?: number & tags.Type<"int32">;
}

export const BarApi = new ConsumptionApi<QueryParams>(
  "bar",
  async (
    { orderBy = "totalRows", limit = 5, startDay = 1, endDay = 31 },
    { client, sql },
  ) => {
    const query = sql`
        SELECT 
          ${BarAggregatedMV.targetTable.columns.dayOfMonth},
          ${BarAggregatedMV.targetTable.columns[orderBy]}
        FROM ${BarAggregatedMV.targetTable}
        WHERE 
          dayOfMonth >= ${startDay} 
          AND dayOfMonth <= ${endDay}
        ORDER BY ${BarAggregatedMV.targetTable.columns[orderBy]} DESC
        LIMIT ${limit}
      `;

    const data = await client.query.execute<{
      dayOfMonth: number;
      totalRows?: number;
      rowsWithText?: number;
      maxTextLength?: number;
      totalTextLength?: number;
    }>(query);

    return data;
  },
);
