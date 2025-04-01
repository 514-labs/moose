"use strict";
var __importDefault =
  (this && this.__importDefault) ||
  function (mod) {
    return mod && mod.__esModule ? mod : { default: mod };
  };
Object.defineProperty(exports, "__esModule", { value: true });
exports.BarAggregatedMV = void 0;
var typia_1 = __importDefault(require("typia"));
var dmv2_1 = require("@514labs/moose-lib/dist/dmv2");
exports.BarAggregatedMV = new dmv2_1.MaterializedView(
  {
    tableName: "BarAggregated",
    materializedViewName: "BarAggregated_MV",
    orderByFields: ["dayOfMonth"],
    selectStatement:
      "SELECT\n    toDayOfMonth(utcTimestamp) as dayOfMonth,\n    count(primaryKey) as totalRows,\n    countIf(hasText) as rowsWithText,\n    sum(textLength) as totalTextLength,\n    max(textLength) as maxTextLength\n  FROM Bar\n  GROUP BY toDayOfMonth(utcTimestamp)\n  ",
  },
  {
    version: "3.1",
    components: {
      schemas: {
        BarAggregated: {
          type: "object",
          properties: {
            dayOfMonth: {
              type: "integer",
            },
            totalRows: {
              type: "integer",
            },
            rowsWithText: {
              type: "integer",
            },
            totalTextLength: {
              type: "integer",
            },
            maxTextLength: {
              type: "integer",
            },
          },
          required: [
            "dayOfMonth",
            "totalRows",
            "rowsWithText",
            "totalTextLength",
            "maxTextLength",
          ],
        },
      },
    },
    schemas: [
      {
        $ref: "#/components/schemas/BarAggregated",
      },
    ],
  },
  JSON.parse(
    '[{"name":"dayOfMonth","data_type":"Int","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"totalRows","data_type":"Int","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"rowsWithText","data_type":"Int","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"totalTextLength","data_type":"Int","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"maxTextLength","data_type":"Int","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]}]',
  ),
);
//# sourceMappingURL=blocks.js.map
