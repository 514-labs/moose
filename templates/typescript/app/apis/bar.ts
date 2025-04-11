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

//Define the data model for the response body from the API endpoint
interface ResponseBody {
  dayOfMonth: number;
  totalRows?: number;
  rowsWithText?: number;
  maxTextLength?: number;
  totalTextLength?: number;
}

export const BarApi = new ConsumptionApi<QueryParams, ResponseBody[]>(
  "BarAggregated",
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

    //Define the query for the API endpoint - plug in the query parameters and data model fields as variables
    const query = sql`
        SELECT 
          ${BACols.dayOfMonth} as dayOfMonth,
          ${BACols[orderBy]}
        FROM BarAggregated
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
