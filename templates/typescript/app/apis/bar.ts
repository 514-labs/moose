import { ConsumptionApi, MooseCache } from "@514labs/moose-lib";
import { BarAggregatedMV } from "../views/barAggregated";
import { tags } from "typia";

// This file is where you can define your APIs to consume your data
interface QueryParams {
  orderBy?: "totalRows" | "rowsWithText" | "maxTextLength" | "totalTextLength";
  limit?: number;
  startDay?: number & tags.Type<"int32">;
  endDay?: number & tags.Type<"int32">;
}

interface ResponseData {
  dayOfMonth: number;
  totalRows?: number;
  rowsWithText?: number;
  maxTextLength?: number;
  totalTextLength?: number;
}

export const BarApi = new ConsumptionApi<QueryParams, any>(
  "bar",
  async (
    { orderBy = "totalRows", limit = 1, startDay = 1, endDay = 31 },
    { client, sql },
  ) => {
    return { version: "1" };
  },
  { version: "1" },
);

export const BarApiV2 = new ConsumptionApi<QueryParams, any>(
  "bar",
  async (
    { orderBy = "totalRows", limit = 20, startDay = 1, endDay = 31 },
    { client, sql },
  ) => {
    return { version: "2" };
  },
  { version: "2" },
);
