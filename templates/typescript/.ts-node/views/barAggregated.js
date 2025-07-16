"use strict";
var __makeTemplateObject =
  (this && this.__makeTemplateObject) ||
  function (cooked, raw) {
    if (Object.defineProperty) {
      Object.defineProperty(cooked, "raw", { value: raw });
    } else {
      cooked.raw = raw;
    }
    return cooked;
  };
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
var barTable = models_1.BarPipeline.table;
var barColumns = barTable.columns;
exports.BarAggregatedMV = new moose_lib_1.MaterializedView(
  {
    tableName: "BarAggregated",
    materializedViewName: "BarAggregated_MV",
    orderByFields: ["dayOfMonth"],
    selectStatement: (0, moose_lib_1.sql)(
      templateObject_1 ||
        (templateObject_1 = __makeTemplateObject(
          [
            "SELECT\n    toDayOfMonth(",
            ") as dayOfMonth,\n    count(",
            ") as totalRows,\n    countIf(",
            ") as rowsWithText,\n    sum(",
            ") as totalTextLength,\n    max(",
            ") as maxTextLength\n  FROM ",
            "\n  GROUP BY toDayOfMonth(utcTimestamp)\n  ",
          ],
          [
            "SELECT\n    toDayOfMonth(",
            ") as dayOfMonth,\n    count(",
            ") as totalRows,\n    countIf(",
            ") as rowsWithText,\n    sum(",
            ") as totalTextLength,\n    max(",
            ") as maxTextLength\n  FROM ",
            "\n  GROUP BY toDayOfMonth(utcTimestamp)\n  ",
          ],
        )),
      barColumns.utcTimestamp,
      barColumns.primaryKey,
      barColumns.hasText,
      barColumns.textLength,
      barColumns.textLength,
      barTable,
    ),
    selectTables: [barTable],
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
    '[{"name":"dayOfMonth","data_type":"Int64","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"totalRows","data_type":"Int64","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"rowsWithText","data_type":"Int64","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"totalTextLength","data_type":"Int64","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"maxTextLength","data_type":"Int64","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]}]',
  ),
);
var templateObject_1;
//# sourceMappingURL=barAggregated.js.map
