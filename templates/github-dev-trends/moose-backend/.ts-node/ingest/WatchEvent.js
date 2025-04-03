"use strict";
var __importDefault =
  (this && this.__importDefault) ||
  function (mod) {
    return mod && mod.__esModule ? mod : { default: mod };
  };
Object.defineProperty(exports, "__esModule", { value: true });
exports.watchEventPipeline = void 0;
var typia_1 = __importDefault(require("typia"));
var moose_lib_1 = require("@514labs/moose-lib");
exports.watchEventPipeline = new moose_lib_1.IngestPipeline(
  "watch-event",
  {
    ingest: true,
    table: true,
    stream: true,
  },
  {
    version: "3.1",
    components: {
      schemas: {
        WatchEventWithRepo: {
          type: "object",
          properties: {
            repoDescription: {
              type: "string",
            },
            repoTopics: {
              type: "array",
              items: {
                type: "string",
              },
            },
            repoLanguage: {
              type: "string",
            },
            repoStars: {
              type: "number",
            },
            repoForks: {
              type: "number",
            },
            repoWatchers: {
              type: "number",
            },
            repoOpenIssues: {
              type: "number",
            },
            repoCreatedAt: {
              type: "string",
              format: "date-time",
            },
            repoOwnerLogin: {
              type: "string",
            },
            repoOwnerId: {
              type: "number",
            },
            repoOwnerUrl: {
              type: "string",
            },
            repoOwnerAvatarUrl: {
              type: "string",
            },
            repoOwnerType: {
              type: "string",
            },
            repoOrgId: {
              type: "number",
            },
            repoOrgUrl: {
              type: "string",
            },
            repoOrgLogin: {
              type: "string",
            },
            repoHomepage: {
              type: "string",
            },
            eventId: {
              type: "string",
            },
            createdAt: {
              type: "string",
              format: "date-time",
            },
            actorLogin: {
              type: "string",
            },
            actorId: {
              type: "number",
            },
            actorUrl: {
              type: "string",
            },
            actorAvatarUrl: {
              type: "string",
            },
            repoName: {
              type: "string",
            },
            repoUrl: {
              type: "string",
            },
            repoId: {
              type: "number",
            },
          },
          required: [
            "eventId",
            "createdAt",
            "actorLogin",
            "actorId",
            "actorUrl",
            "actorAvatarUrl",
            "repoName",
            "repoUrl",
            "repoId",
          ],
        },
      },
    },
    schemas: [
      {
        $ref: "#/components/schemas/WatchEventWithRepo",
      },
    ],
  },
  JSON.parse(
    '[{"name":"repoDescription","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoTopics","data_type":{"elementNullable":false,"elementType":"String"},"primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoLanguage","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoStars","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoForks","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoWatchers","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoOpenIssues","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoCreatedAt","data_type":"DateTime","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoOwnerLogin","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoOwnerId","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoOwnerUrl","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoOwnerAvatarUrl","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoOwnerType","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoOrgId","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoOrgUrl","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoOrgLogin","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"repoHomepage","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"eventId","data_type":"String","primary_key":true,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"createdAt","data_type":"DateTime","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"actorLogin","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"actorId","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"actorUrl","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"actorAvatarUrl","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"repoName","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"repoUrl","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"repoId","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]}]',
  ),
);
//# sourceMappingURL=WatchEvent.js.map
