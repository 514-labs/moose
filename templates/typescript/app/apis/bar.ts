import { ConsumptionApi, ConsumptionHelpers as CH } from "@514labs/moose-lib";
import { BarAggregatedMV } from "../views/views";
import { tags } from "typia";

// Define the query parameters for the GET API endpoint
interface QueryParams {
  orderBy?: "totalRows" | "rowsWithText" | "maxTextLength" | "totalTextLength";
  limit?: number;
  startDay?: number & tags.Type<"int32">;
  endDay?: number & tags.Type<"int32">;
}

interface ResponseBody {
  dayOfMonth: number;
  totalRows?: number;
  rowsWithText?: number;
  maxTextLength?: number;
  totalTextLength?: number;
}

export const BarApi = new ConsumptionApi<QueryParams, ResponseBody[]>(
  "bar",
  async (
    {
      orderBy = "totalRows",
      limit = 5,
      startDay = 1,
      endDay = 31,
    }: QueryParams,
    { client, sql },
  ) => {
    const BACols = BarAggregatedMV.targetTable!.columns;

    const query = sql`
        SELECT 
          ${BACols.dayOfMonth} as dayOfMonth,
          ${BACols[orderBy]}
        FROM ${BarAggregatedMV.targetTable.name}
        WHERE 
          dayOfMonth >= ${startDay} 
          AND dayOfMonth <= ${endDay}
        ORDER BY ${BACols[orderBy]} DESC
        LIMIT ${limit}
      `;

    const data = await client.query.execute(query);

    return (await data.json()) as ResponseBody[];
  },
);
