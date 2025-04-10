"use strict";
var __importDefault =
  (this && this.__importDefault) ||
  function (mod) {
    return mod && mod.__esModule ? mod : { default: mod };
  };
Object.defineProperty(exports, "__esModule", { value: true });
exports.BarPipeline = exports.FooPipeline = void 0;
var typia_1 = __importDefault(require("typia"));
var moose_lib_1 = require("@514labs/moose-lib");
//Automatically generate an ingest pipeline typed to "Foo"
//Includes an API endpoint and a streaming buffer - but no landing table
exports.FooPipeline = new moose_lib_1.IngestPipeline(
  "Foo",
  {
    table: false,
    stream: true,
    ingest: true,
  },
  {
    version: "3.1",
    components: {
      schemas: {
        Foo: {
          type: "object",
          properties: {
            primaryKey: {
              type: "string",
            },
            timestamp: {
              type: "number",
            },
            optionalText: {
              type: "string",
            },
          },
          required: ["primaryKey", "timestamp"],
        },
      },
    },
    schemas: [
      {
        $ref: "#/components/schemas/Foo",
      },
    ],
  },
  JSON.parse(
    '[{"name":"primaryKey","data_type":"String","primary_key":true,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"timestamp","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"optionalText","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]}]',
  ),
);
//Automatically generate an ingest pipeline typed to "Bar" (a downstream data model from "Foo")
//Includes a streaming buffer and a landing table, but no API endpoint
exports.BarPipeline = new moose_lib_1.IngestPipeline(
  "Bar",
  {
    table: true,
    stream: true,
    ingest: false,
  },
  {
    version: "3.1",
    components: {
      schemas: {
        Bar: {
          type: "object",
          properties: {
            primaryKey: {
              type: "string",
            },
            utcTimestamp: {
              type: "string",
              format: "date-time",
            },
            hasText: {
              type: "boolean",
            },
            textLength: {
              type: "number",
            },
          },
          required: ["primaryKey", "utcTimestamp", "hasText", "textLength"],
        },
      },
    },
    schemas: [
      {
        $ref: "#/components/schemas/Bar",
      },
    ],
  },
  JSON.parse(
    '[{"name":"primaryKey","data_type":"String","primary_key":true,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"utcTimestamp","data_type":"DateTime","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"hasText","data_type":"Boolean","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"textLength","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]}]',
  ),
);
//# sourceMappingURL=models.js.map
