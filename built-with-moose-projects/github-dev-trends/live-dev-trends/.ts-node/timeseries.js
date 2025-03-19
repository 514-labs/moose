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
var __createBinding =
  (this && this.__createBinding) ||
  (Object.create
    ? function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        var desc = Object.getOwnPropertyDescriptor(m, k);
        if (
          !desc ||
          ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)
        ) {
          desc = {
            enumerable: true,
            get: function () {
              return m[k];
            },
          };
        }
        Object.defineProperty(o, k2, desc);
      }
    : function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        o[k2] = m[k];
      });
var __setModuleDefault =
  (this && this.__setModuleDefault) ||
  (Object.create
    ? function (o, v) {
        Object.defineProperty(o, "default", { enumerable: true, value: v });
      }
    : function (o, v) {
        o["default"] = v;
      });
var __importStar =
  (this && this.__importStar) ||
  (function () {
    var ownKeys = function (o) {
      ownKeys =
        Object.getOwnPropertyNames ||
        function (o) {
          var ar = [];
          for (var k in o)
            if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
          return ar;
        };
      return ownKeys(o);
    };
    return function (mod) {
      if (mod && mod.__esModule) return mod;
      var result = {};
      if (mod != null)
        for (var k = ownKeys(mod), i = 0; i < k.length; i++)
          if (k[i] !== "default") __createBinding(result, mod, k[i]);
      __setModuleDefault(result, mod);
      return result;
    };
  })();
var __awaiter =
  (this && this.__awaiter) ||
  function (thisArg, _arguments, P, generator) {
    function adopt(value) {
      return value instanceof P
        ? value
        : new P(function (resolve) {
            resolve(value);
          });
    }
    return new (P || (P = Promise))(function (resolve, reject) {
      function fulfilled(value) {
        try {
          step(generator.next(value));
        } catch (e) {
          reject(e);
        }
      }
      function rejected(value) {
        try {
          step(generator["throw"](value));
        } catch (e) {
          reject(e);
        }
      }
      function step(result) {
        result.done
          ? resolve(result.value)
          : adopt(result.value).then(fulfilled, rejected);
      }
      step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
  };
var __generator =
  (this && this.__generator) ||
  function (thisArg, body) {
    var _ = {
        label: 0,
        sent: function () {
          if (t[0] & 1) throw t[1];
          return t[1];
        },
        trys: [],
        ops: [],
      },
      f,
      y,
      t,
      g = Object.create(
        (typeof Iterator === "function" ? Iterator : Object).prototype,
      );
    return (
      (g.next = verb(0)),
      (g["throw"] = verb(1)),
      (g["return"] = verb(2)),
      typeof Symbol === "function" &&
        (g[Symbol.iterator] = function () {
          return this;
        }),
      g
    );
    function verb(n) {
      return function (v) {
        return step([n, v]);
      };
    }
    function step(op) {
      if (f) throw new TypeError("Generator is already executing.");
      while ((g && ((g = 0), op[0] && (_ = 0)), _))
        try {
          if (
            ((f = 1),
            y &&
              (t =
                op[0] & 2
                  ? y["return"]
                  : op[0]
                    ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                    : y.next) &&
              !(t = t.call(y, op[1])).done)
          )
            return t;
          if (((y = 0), t)) op = [op[0] & 2, t.value];
          switch (op[0]) {
            case 0:
            case 1:
              t = op;
              break;
            case 4:
              _.label++;
              return { value: op[1], done: false };
            case 5:
              _.label++;
              y = op[1];
              op = [0];
              continue;
            case 7:
              op = _.ops.pop();
              _.trys.pop();
              continue;
            default:
              if (
                !((t = _.trys), (t = t.length > 0 && t[t.length - 1])) &&
                (op[0] === 6 || op[0] === 2)
              ) {
                _ = 0;
                continue;
              }
              if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                _.label = op[1];
                break;
              }
              if (op[0] === 6 && _.label < t[1]) {
                _.label = t[1];
                t = op;
                break;
              }
              if (t && _.label < t[2]) {
                _.label = t[2];
                _.ops.push(op);
                break;
              }
              if (t[2]) _.ops.pop();
              _.trys.pop();
              continue;
          }
          op = body.call(thisArg, _);
        } catch (e) {
          op = [6, e];
          y = 0;
        } finally {
          f = t = 0;
        }
      if (op[0] & 5) throw op[1];
      return { value: op[0] ? op[1] : void 0, done: true };
    }
  };
var __importDefault =
  (this && this.__importDefault) ||
  function (mod) {
    return mod && mod.__esModule ? mod : { default: mod };
  };
Object.defineProperty(exports, "__esModule", { value: true });
var __typia_transform__isTypeInt32 = __importStar(
  require("typia/lib/internal/_isTypeInt32.js"),
);
var __typia_transform__assertGuard = __importStar(
  require("typia/lib/internal/_assertGuard.js"),
);
var __typia_transform__httpQueryParseURLSearchParams = __importStar(
  require("typia/lib/internal/_httpQueryParseURLSearchParams.js"),
);
var __typia_transform__httpQueryReadString = __importStar(
  require("typia/lib/internal/_httpQueryReadString.js"),
);
var __typia_transform__httpQueryReadNumber = __importStar(
  require("typia/lib/internal/_httpQueryReadNumber.js"),
);
var typia_1 = __importDefault(require("typia"));
exports.default = (function () {
  var assertGuard = (function () {
    var _io0 = function (input) {
      return (
        (undefined === input.interval ||
          "hourly" === input.interval ||
          "daily" === input.interval ||
          "weekly" === input.interval) &&
        (undefined === input.limit ||
          ("number" === typeof input.limit &&
            10 <= input.limit &&
            __typia_transform__isTypeInt32._isTypeInt32(input.limit))) &&
        (undefined === input.exclude ||
          ("string" === typeof input.exclude &&
            RegExp("^([^,]+)(,[^,]+)*$").test(input.exclude)))
      );
    };
    var _ao0 = function (input, _path, _exceptionable) {
      if (_exceptionable === void 0) {
        _exceptionable = true;
      }
      return (
        (undefined === input.interval ||
          "hourly" === input.interval ||
          "daily" === input.interval ||
          "weekly" === input.interval ||
          __typia_transform__assertGuard._assertGuard(
            _exceptionable,
            {
              method: "____moose____typia.http.createAssertQuery",
              path: _path + ".interval",
              expected: '("daily" | "hourly" | "weekly" | undefined)',
              value: input.interval,
            },
            _errorFactory,
          )) &&
        (undefined === input.limit ||
          ("number" === typeof input.limit &&
            (10 <= input.limit ||
              __typia_transform__assertGuard._assertGuard(
                _exceptionable,
                {
                  method: "____moose____typia.http.createAssertQuery",
                  path: _path + ".limit",
                  expected: "number & Minimum<10>",
                  value: input.limit,
                },
                _errorFactory,
              )) &&
            (__typia_transform__isTypeInt32._isTypeInt32(input.limit) ||
              __typia_transform__assertGuard._assertGuard(
                _exceptionable,
                {
                  method: "____moose____typia.http.createAssertQuery",
                  path: _path + ".limit",
                  expected: 'number & Type<"int32">',
                  value: input.limit,
                },
                _errorFactory,
              ))) ||
          __typia_transform__assertGuard._assertGuard(
            _exceptionable,
            {
              method: "____moose____typia.http.createAssertQuery",
              path: _path + ".limit",
              expected: '((number & Minimum<10> & Type<"int32">) | undefined)',
              value: input.limit,
            },
            _errorFactory,
          )) &&
        (undefined === input.exclude ||
          ("string" === typeof input.exclude &&
            (RegExp("^([^,]+)(,[^,]+)*$").test(input.exclude) ||
              __typia_transform__assertGuard._assertGuard(
                _exceptionable,
                {
                  method: "____moose____typia.http.createAssertQuery",
                  path: _path + ".exclude",
                  expected: 'string & Pattern<"^([^,]+)(,[^,]+)*$">',
                  value: input.exclude,
                },
                _errorFactory,
              ))) ||
          __typia_transform__assertGuard._assertGuard(
            _exceptionable,
            {
              method: "____moose____typia.http.createAssertQuery",
              path: _path + ".exclude",
              expected:
                '((string & Pattern<"^([^,]+)(,[^,]+)*$">) | undefined)',
              value: input.exclude,
            },
            _errorFactory,
          ))
      );
    };
    var __is = function (input) {
      return (
        "object" === typeof input &&
        null !== input &&
        false === Array.isArray(input) &&
        _io0(input)
      );
    };
    var _errorFactory;
    var __assert = function (input, errorFactory) {
      if (false === __is(input)) {
        _errorFactory = errorFactory;
        (function (input, _path, _exceptionable) {
          if (_exceptionable === void 0) {
            _exceptionable = true;
          }
          return (
            ((("object" === typeof input &&
              null !== input &&
              false === Array.isArray(input)) ||
              __typia_transform__assertGuard._assertGuard(
                true,
                {
                  method: "____moose____typia.http.createAssertQuery",
                  path: _path + "",
                  expected: "QueryParams",
                  value: input,
                },
                _errorFactory,
              )) &&
              _ao0(input, _path + "", true)) ||
            __typia_transform__assertGuard._assertGuard(
              true,
              {
                method: "____moose____typia.http.createAssertQuery",
                path: _path + "",
                expected: "QueryParams",
                value: input,
              },
              _errorFactory,
            )
          );
        })(input, "$input", true);
      }
      return input;
    };
    var __decode = function (input) {
      var _a, _b, _c;
      input =
        __typia_transform__httpQueryParseURLSearchParams._httpQueryParseURLSearchParams(
          input,
        );
      var output = {
        interval:
          (_a = __typia_transform__httpQueryReadString._httpQueryReadString(
            input.get("interval"),
          )) !== null && _a !== void 0
            ? _a
            : undefined,
        limit:
          (_b = __typia_transform__httpQueryReadNumber._httpQueryReadNumber(
            input.get("limit"),
          )) !== null && _b !== void 0
            ? _b
            : undefined,
        exclude:
          (_c = __typia_transform__httpQueryReadString._httpQueryReadString(
            input.get("exclude"),
          )) !== null && _c !== void 0
            ? _c
            : undefined,
      };
      return output;
    };
    return function (input, errorFactory) {
      return __assert(__decode(input), errorFactory);
    };
  })();
  var handlerFunc = function (_a, _b) {
    return __awaiter(void 0, [_a, _b], void 0, function (_c, _d) {
      var intervalMap, query, result;
      var _e = _c.interval,
        interval = _e === void 0 ? "hourly" : _e,
        _f = _c.limit,
        limit = _f === void 0 ? 10 : _f,
        _g = _c.exclude,
        exclude = _g === void 0 ? "" : _g;
      var client = _d.client,
        sql = _d.sql;
      return __generator(this, function (_h) {
        switch (_h.label) {
          case 0:
            intervalMap = {
              hourly: {
                select: sql(
                  templateObject_1 ||
                    (templateObject_1 = __makeTemplateObject(
                      ["toStartOfHour(created_at) AS hour"],
                      ["toStartOfHour(created_at) AS hour"],
                    )),
                ),
                groupBy: sql(
                  templateObject_2 ||
                    (templateObject_2 = __makeTemplateObject(
                      ["GROUP BY hour, topic"],
                      ["GROUP BY hour, topic"],
                    )),
                ),
                orderBy: sql(
                  templateObject_3 ||
                    (templateObject_3 = __makeTemplateObject(
                      ["ORDER BY hour, total_events DESC"],
                      ["ORDER BY hour, total_events DESC"],
                    )),
                ),
                limit: sql(
                  templateObject_4 ||
                    (templateObject_4 = __makeTemplateObject(
                      ["LIMIT ", " BY hour"],
                      ["LIMIT ", " BY hour"],
                    )),
                  limit,
                ),
              },
              daily: {
                select: sql(
                  templateObject_5 ||
                    (templateObject_5 = __makeTemplateObject(
                      ["toStartOfDay(created_at) AS day"],
                      ["toStartOfDay(created_at) AS day"],
                    )),
                ),
                groupBy: sql(
                  templateObject_6 ||
                    (templateObject_6 = __makeTemplateObject(
                      ["GROUP BY day, topic"],
                      ["GROUP BY day, topic"],
                    )),
                ),
                orderBy: sql(
                  templateObject_7 ||
                    (templateObject_7 = __makeTemplateObject(
                      ["ORDER BY day, total_events DESC"],
                      ["ORDER BY day, total_events DESC"],
                    )),
                ),
                limit: sql(
                  templateObject_8 ||
                    (templateObject_8 = __makeTemplateObject(
                      ["LIMIT ", " BY day"],
                      ["LIMIT ", " BY day"],
                    )),
                  limit,
                ),
              },
              weekly: {
                select: sql(
                  templateObject_9 ||
                    (templateObject_9 = __makeTemplateObject(
                      ["toStartOfWeek(created_at) AS week"],
                      ["toStartOfWeek(created_at) AS week"],
                    )),
                ),
                groupBy: sql(
                  templateObject_10 ||
                    (templateObject_10 = __makeTemplateObject(
                      ["GROUP BY week, topic"],
                      ["GROUP BY week, topic"],
                    )),
                ),
                orderBy: sql(
                  templateObject_11 ||
                    (templateObject_11 = __makeTemplateObject(
                      ["ORDER BY week, total_events DESC"],
                      ["ORDER BY week, total_events DESC"],
                    )),
                ),
                limit: sql(
                  templateObject_12 ||
                    (templateObject_12 = __makeTemplateObject(
                      ["LIMIT ", " BY week"],
                      ["LIMIT ", " BY week"],
                    )),
                  limit,
                ),
              },
            };
            query = sql(
              templateObject_15 ||
                (templateObject_15 = __makeTemplateObject(
                  [
                    "\n            SELECT\n                ",
                    ",\n                arrayJoin(repo_topics) AS topic,\n                count() AS total_events,\n                uniqExact(repo_id) AS unique_repos_count,\n                uniqExact(actor_id) AS unique_users_count\n            FROM StarEvent_0_0\n            WHERE length(repo_topics) > 0\n            ",
                    "\n            ",
                    "\n            ",
                    "\n            ",
                    ";\n        ",
                  ],
                  [
                    "\n            SELECT\n                ",
                    ",\n                arrayJoin(repo_topics) AS topic,\n                count() AS total_events,\n                uniqExact(repo_id) AS unique_repos_count,\n                uniqExact(actor_id) AS unique_users_count\n            FROM StarEvent_0_0\n            WHERE length(repo_topics) > 0\n            ",
                    "\n            ",
                    "\n            ",
                    "\n            ",
                    ";\n        ",
                  ],
                )),
              intervalMap[interval].select,
              exclude
                ? sql(
                    templateObject_13 ||
                      (templateObject_13 = __makeTemplateObject(
                        ["AND arrayAll(x -> x NOT IN (", "), repo_topics)"],
                        ["AND arrayAll(x -> x NOT IN (", "), repo_topics)"],
                      )),
                    exclude,
                  )
                : sql(
                    templateObject_14 ||
                      (templateObject_14 = __makeTemplateObject([""], [""])),
                  ),
              intervalMap[interval].groupBy,
              intervalMap[interval].orderBy,
              intervalMap[interval].limit,
            );
            return [4 /*yield*/, client.query.execute(query)];
          case 1:
            result = _h.sent();
            return [2 /*return*/, result];
        }
      });
    });
  };
  var wrappedFunc = function (params, utils) {
    var processedParams = assertGuard(new URLSearchParams(params));
    return handlerFunc(processedParams, utils);
  };
  wrappedFunc["moose_input_schema"] = {
    version: "3.1",
    components: {
      schemas: {
        QueryParams: {
          type: "object",
          properties: {
            interval: {
              oneOf: [
                {
                  const: "hourly",
                },
                {
                  const: "daily",
                },
                {
                  const: "weekly",
                },
              ],
            },
            limit: {
              type: "integer",
              minimum: 10,
            },
            exclude: {
              type: "string",
              pattern: "^([^,]+)(,[^,]+)*$",
            },
          },
          required: [],
        },
      },
    },
    schemas: [
      {
        $ref: "#/components/schemas/QueryParams",
      },
    ],
  };
  wrappedFunc["moose_output_schema"] = {
    version: "3.1",
    components: {
      schemas: {},
    },
    schemas: [
      {
        type: "array",
        items: {
          type: "object",
          properties: {
            hour: {
              type: "string",
            },
            day: {
              type: "string",
            },
            week: {
              type: "string",
            },
            topic: {
              type: "string",
            },
            total_events: {
              type: "number",
            },
            unique_repos_count: {
              type: "number",
            },
            unique_users_count: {
              type: "number",
            },
          },
          required: [
            "topic",
            "total_events",
            "unique_repos_count",
            "unique_users_count",
          ],
        },
      },
    ],
  };
  return wrappedFunc;
})();
var templateObject_1,
  templateObject_2,
  templateObject_3,
  templateObject_4,
  templateObject_5,
  templateObject_6,
  templateObject_7,
  templateObject_8,
  templateObject_9,
  templateObject_10,
  templateObject_11,
  templateObject_12,
  templateObject_13,
  templateObject_14,
  templateObject_15;
//# sourceMappingURL=timeseries.js.map
