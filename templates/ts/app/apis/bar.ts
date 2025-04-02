import { ConsumptionHelpers as CH, ConsumptionApi } from "@514labs/moose-lib";
import { tags } from "typia";

// This file is where you can define your APIs to consume your data
interface QueryParams {
  orderBy: "totalRows" | "rowsWithText" | "maxTextLength" | "totalTextLength";
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
          dayOfMonth,
          ${CH.column(orderBy)}
        FROM BarAggregated_MV
        WHERE 
          dayOfMonth >= ${startDay} 
          AND dayOfMonth <= ${endDay}
        ORDER BY ${CH.column(orderBy)} DESC
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
