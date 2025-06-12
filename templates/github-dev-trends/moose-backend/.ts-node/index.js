"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.topicTimeseriesApi = exports.RepoStarEvent = exports.GhEvent = void 0;
var __typia_transform__isTypeInt32 = __importStar(require("typia/lib/internal/_isTypeInt32.js"));
var __typia_transform__assertGuard = __importStar(require("typia/lib/internal/_assertGuard.js"));
var __typia_transform__httpQueryParseURLSearchParams = __importStar(require("typia/lib/internal/_httpQueryParseURLSearchParams.js"));
var __typia_transform__httpQueryReadString = __importStar(require("typia/lib/internal/_httpQueryReadString.js"));
var __typia_transform__httpQueryReadNumber = __importStar(require("typia/lib/internal/_httpQueryReadNumber.js"));
var typia_1 = __importDefault(require("typia"));
var transform_1 = require("./ingest/transform");
var topicTimeseries_1 = require("./apis/topicTimeseries");
var moose_lib_1 = require("@514labs/moose-lib");
// Pipeline to receive raw events from the Github API
exports.GhEvent = new moose_lib_1.IngestPipeline("GhEvent", {
    ingest: true,
    table: true,
    stream: true,
}, {
    version: "3.1",
    components: {
        schemas: {
            IGhEvent: {
                type: "object",
                properties: {
                    eventType: {
                        type: "string"
                    },
                    eventId: {
                        type: "string"
                    },
                    createdAt: {
                        type: "string",
                        format: "date-time"
                    },
                    actorLogin: {
                        type: "string"
                    },
                    actorId: {
                        type: "number"
                    },
                    actorUrl: {
                        type: "string"
                    },
                    actorAvatarUrl: {
                        type: "string"
                    },
                    repoUrl: {
                        type: "string"
                    },
                    repoId: {
                        type: "number"
                    },
                    repoOwner: {
                        type: "string"
                    },
                    repoName: {
                        type: "string"
                    },
                    repoFullName: {
                        type: "string"
                    }
                },
                required: [
                    "eventType",
                    "eventId",
                    "createdAt",
                    "actorLogin",
                    "actorId",
                    "actorUrl",
                    "actorAvatarUrl",
                    "repoUrl",
                    "repoId",
                    "repoOwner",
                    "repoName",
                    "repoFullName"
                ]
            }
        }
    },
    schemas: [
        {
            $ref: "#/components/schemas/IGhEvent"
        }
    ]
}, JSON.parse("[{\"name\":\"eventType\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"eventId\",\"data_type\":\"String\",\"primary_key\":true,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"createdAt\",\"data_type\":\"DateTime\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"actorLogin\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"actorId\",\"data_type\":\"Float\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"actorUrl\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"actorAvatarUrl\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoUrl\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoId\",\"data_type\":\"Float\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoOwner\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoName\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoFullName\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]}]"));
// Pipeline to receive transformed events from the GhEvent pipeline
exports.RepoStarEvent = new moose_lib_1.IngestPipeline("RepoStar", {
    ingest: false,
    stream: true,
    table: true,
}, {
    version: "3.1",
    components: {
        schemas: {
            IRepoStarEvent: {
                type: "object",
                properties: {
                    repoDescription: {
                        type: "string"
                    },
                    repoTopics: {
                        type: "array",
                        items: {
                            type: "string"
                        }
                    },
                    repoLanguage: {
                        type: "string"
                    },
                    repoStars: {
                        type: "number"
                    },
                    repoForks: {
                        type: "number"
                    },
                    repoWatchers: {
                        type: "number"
                    },
                    repoOpenIssues: {
                        type: "number"
                    },
                    repoCreatedAt: {
                        type: "string",
                        format: "date-time"
                    },
                    repoOwnerLogin: {
                        type: "string"
                    },
                    repoOwnerId: {
                        type: "number"
                    },
                    repoOwnerUrl: {
                        type: "string"
                    },
                    repoOwnerAvatarUrl: {
                        type: "string"
                    },
                    repoOwnerType: {
                        type: "string"
                    },
                    repoOrgId: {
                        type: "number"
                    },
                    repoOrgUrl: {
                        type: "string"
                    },
                    repoOrgLogin: {
                        type: "string"
                    },
                    repoHomepage: {
                        type: "string"
                    },
                    eventType: {
                        type: "string"
                    },
                    eventId: {
                        type: "string"
                    },
                    createdAt: {
                        type: "string",
                        format: "date-time"
                    },
                    actorLogin: {
                        type: "string"
                    },
                    actorId: {
                        type: "number"
                    },
                    actorUrl: {
                        type: "string"
                    },
                    actorAvatarUrl: {
                        type: "string"
                    },
                    repoUrl: {
                        type: "string"
                    },
                    repoId: {
                        type: "number"
                    },
                    repoOwner: {
                        type: "string"
                    },
                    repoName: {
                        type: "string"
                    },
                    repoFullName: {
                        type: "string"
                    }
                },
                required: [
                    "repoDescription",
                    "repoTopics",
                    "repoLanguage",
                    "repoStars",
                    "repoForks",
                    "repoWatchers",
                    "repoOpenIssues",
                    "repoCreatedAt",
                    "repoOwnerLogin",
                    "repoOwnerId",
                    "repoOwnerUrl",
                    "repoOwnerAvatarUrl",
                    "repoOwnerType",
                    "repoOrgId",
                    "repoOrgUrl",
                    "repoOrgLogin",
                    "repoHomepage",
                    "eventType",
                    "eventId",
                    "createdAt",
                    "actorLogin",
                    "actorId",
                    "actorUrl",
                    "actorAvatarUrl",
                    "repoUrl",
                    "repoId",
                    "repoOwner",
                    "repoName",
                    "repoFullName"
                ]
            }
        }
    },
    schemas: [
        {
            $ref: "#/components/schemas/IRepoStarEvent"
        }
    ]
}, JSON.parse("[{\"name\":\"repoDescription\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoTopics\",\"data_type\":{\"elementNullable\":false,\"elementType\":\"String\"},\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoLanguage\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoStars\",\"data_type\":\"Float\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoForks\",\"data_type\":\"Float\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoWatchers\",\"data_type\":\"Float\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoOpenIssues\",\"data_type\":\"Float\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoCreatedAt\",\"data_type\":\"DateTime\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoOwnerLogin\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoOwnerId\",\"data_type\":\"Float\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoOwnerUrl\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoOwnerAvatarUrl\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoOwnerType\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoOrgId\",\"data_type\":\"Float\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoOrgUrl\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoOrgLogin\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoHomepage\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"eventType\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"eventId\",\"data_type\":\"String\",\"primary_key\":true,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"createdAt\",\"data_type\":\"DateTime\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"actorLogin\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"actorId\",\"data_type\":\"Float\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"actorUrl\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"actorAvatarUrl\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoUrl\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoId\",\"data_type\":\"Float\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoOwner\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoName\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"repoFullName\",\"data_type\":\"String\",\"primary_key\":false,\"required\":true,\"unique\":false,\"default\":null,\"annotations\":[]}]"));
// Streaming transformation to transform the raw GhEvent stream into an enriched RepoStarEvent stream
exports.GhEvent.stream.addTransform(exports.RepoStarEvent.stream, transform_1.transformGhEvent);
// Consumption API to get a timeseries of the top `n` topics over a given interval
exports.topicTimeseriesApi = new moose_lib_1.ConsumptionApi("topicTimeseries", function (params, utils) {
    var assertGuard = (function () { var _io0 = function (input) { return (undefined === input.interval || "minute" === input.interval || "hour" === input.interval || "day" === input.interval) && (undefined === input.limit || "number" === typeof input.limit && (1 <= input.limit && __typia_transform__isTypeInt32._isTypeInt32(input.limit))) && (undefined === input.exclude || "string" === typeof input.exclude && RegExp("^([^,]+)(,[^,]+)*$").test(input.exclude)); }; var _ao0 = function (input, _path, _exceptionable) {
        if (_exceptionable === void 0) { _exceptionable = true; }
        return (undefined === input.interval || "minute" === input.interval || "hour" === input.interval || "day" === input.interval || __typia_transform__assertGuard._assertGuard(_exceptionable, {
            method: "____moose____typia.http.createAssertQuery",
            path: _path + ".interval",
            expected: "(\"day\" | \"hour\" | \"minute\" | undefined)",
            value: input.interval
        }, _errorFactory)) && (undefined === input.limit || "number" === typeof input.limit && (1 <= input.limit || __typia_transform__assertGuard._assertGuard(_exceptionable, {
            method: "____moose____typia.http.createAssertQuery",
            path: _path + ".limit",
            expected: "number & Minimum<1>",
            value: input.limit
        }, _errorFactory)) && (__typia_transform__isTypeInt32._isTypeInt32(input.limit) || __typia_transform__assertGuard._assertGuard(_exceptionable, {
            method: "____moose____typia.http.createAssertQuery",
            path: _path + ".limit",
            expected: "number & Type<\"int32\">",
            value: input.limit
        }, _errorFactory)) || __typia_transform__assertGuard._assertGuard(_exceptionable, {
            method: "____moose____typia.http.createAssertQuery",
            path: _path + ".limit",
            expected: "((number & Minimum<1> & Type<\"int32\">) | undefined)",
            value: input.limit
        }, _errorFactory)) && (undefined === input.exclude || "string" === typeof input.exclude && (RegExp("^([^,]+)(,[^,]+)*$").test(input.exclude) || __typia_transform__assertGuard._assertGuard(_exceptionable, {
            method: "____moose____typia.http.createAssertQuery",
            path: _path + ".exclude",
            expected: "string & Pattern<\"^([^,]+)(,[^,]+)*$\">",
            value: input.exclude
        }, _errorFactory)) || __typia_transform__assertGuard._assertGuard(_exceptionable, {
            method: "____moose____typia.http.createAssertQuery",
            path: _path + ".exclude",
            expected: "((string & Pattern<\"^([^,]+)(,[^,]+)*$\">) | undefined)",
            value: input.exclude
        }, _errorFactory));
    }; var __is = function (input) { return "object" === typeof input && null !== input && false === Array.isArray(input) && _io0(input); }; var _errorFactory; var __assert = function (input, errorFactory) {
        if (false === __is(input)) {
            _errorFactory = errorFactory;
            (function (input, _path, _exceptionable) {
                if (_exceptionable === void 0) { _exceptionable = true; }
                return ("object" === typeof input && null !== input && false === Array.isArray(input) || __typia_transform__assertGuard._assertGuard(true, {
                    method: "____moose____typia.http.createAssertQuery",
                    path: _path + "",
                    expected: "QueryParams",
                    value: input
                }, _errorFactory)) && _ao0(input, _path + "", true) || __typia_transform__assertGuard._assertGuard(true, {
                    method: "____moose____typia.http.createAssertQuery",
                    path: _path + "",
                    expected: "QueryParams",
                    value: input
                }, _errorFactory);
            })(input, "$input", true);
        }
        return input;
    }; var __decode = function (input) {
        var _a, _b, _c;
        input = __typia_transform__httpQueryParseURLSearchParams._httpQueryParseURLSearchParams(input);
        var output = {
            interval: (_a = __typia_transform__httpQueryReadString._httpQueryReadString(input.get("interval"))) !== null && _a !== void 0 ? _a : undefined,
            limit: (_b = __typia_transform__httpQueryReadNumber._httpQueryReadNumber(input.get("limit"))) !== null && _b !== void 0 ? _b : undefined,
            exclude: (_c = __typia_transform__httpQueryReadString._httpQueryReadString(input.get("exclude"))) !== null && _c !== void 0 ? _c : undefined
        };
        return output;
    }; return function (input, errorFactory) { return __assert(__decode(input), errorFactory); }; })();
    var searchParams = new URLSearchParams(params);
    var processedParams = assertGuard(searchParams);
    return (topicTimeseries_1.getTopicTimeseries)(processedParams, utils);
}, {}, {
    version: "3.1",
    components: {
        schemas: {
            QueryParams: {
                type: "object",
                properties: {
                    interval: {
                        oneOf: [
                            {
                                "const": "minute"
                            },
                            {
                                "const": "hour"
                            },
                            {
                                "const": "day"
                            }
                        ]
                    },
                    limit: {
                        type: "integer",
                        minimum: 1
                    },
                    exclude: {
                        type: "string",
                        pattern: "^([^,]+)(,[^,]+)*$"
                    }
                },
                required: []
            }
        }
    },
    schemas: [
        {
            $ref: "#/components/schemas/QueryParams"
        }
    ]
}, JSON.parse("[{\"name\":\"interval\",\"data_type\":\"String\",\"primary_key\":false,\"required\":false,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"limit\",\"data_type\":\"Float\",\"primary_key\":false,\"required\":false,\"unique\":false,\"default\":null,\"annotations\":[]},{\"name\":\"exclude\",\"data_type\":\"String\",\"primary_key\":false,\"required\":false,\"unique\":false,\"default\":null,\"annotations\":[]}]"), {
    version: "3.1",
    components: {
        schemas: {
            ResponseBody: {
                type: "object",
                properties: {
                    time: {
                        type: "string"
                    },
                    topicStats: {
                        type: "array",
                        items: {
                            $ref: "#/components/schemas/TopicStats"
                        }
                    }
                },
                required: [
                    "time",
                    "topicStats"
                ]
            },
            TopicStats: {
                type: "object",
                properties: {
                    topic: {
                        type: "string"
                    },
                    eventCount: {
                        type: "number"
                    },
                    uniqueRepos: {
                        type: "number"
                    },
                    uniqueUsers: {
                        type: "number"
                    }
                },
                required: [
                    "topic",
                    "eventCount",
                    "uniqueRepos",
                    "uniqueUsers"
                ]
            }
        }
    },
    schemas: [
        {
            type: "array",
            items: {
                $ref: "#/components/schemas/ResponseBody"
            }
        }
    ]
});
//# sourceMappingURL=index.js.map