"use strict";
var __importDefault =
  (this && this.__importDefault) ||
  function (mod) {
    return mod && mod.__esModule ? mod : { default: mod };
  };
Object.defineProperty(exports, "__esModule", { value: true });
exports.BarAggregatedMV = void 0;
var typia_1 = __importDefault(require("typia"));
var moose_lib_1 = require("@514labs/moose-lib");
var models_1 = require("../ingest/models");
var BTCols = models_1.BarPipeline.table.columns;
var query = "SELECT\n    toDayOfMonth("
  .concat(BTCols.utcTimestamp.name, ") as dayOfMonth,\n    count(")
  .concat(BTCols.primaryKey.name, ") as totalRows,\n    countIf(")
  .concat(BTCols.hasText.name, ") as rowsWithText,\n    sum(")
  .concat(BTCols.textLength.name, ") as totalTextLength,\n    max(")
  .concat(BTCols.textLength.name, ") as maxTextLength\n  FROM ")
  .concat(models_1.BarPipeline.table.name, "\n  GROUP BY toDayOfMonth(")
  .concat(BTCols.utcTimestamp.name, ")\n  ");
exports.BarAggregatedMV = new moose_lib_1.MaterializedView(
  {
    tableName: "BarAggregated",
    materializedViewName: "BarAggregated_MV",
    orderByFields: ["dayOfMonth"],
    selectStatement: query,
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
//# sourceMappingURL=views.js.map
