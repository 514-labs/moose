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
exports.BarApi = void 0;
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
var moose_lib_1 = require("@514labs/moose-lib");
var views_1 = require("../views/views");
exports.BarApi = new moose_lib_1.ConsumptionApi(
  "bar",
  function (params, utils) {
    var assertGuard = (function () {
      var _io0 = function (input) {
        return (
          (undefined === input.orderBy ||
            "totalRows" === input.orderBy ||
            "rowsWithText" === input.orderBy ||
            "maxTextLength" === input.orderBy ||
            "totalTextLength" === input.orderBy) &&
          (undefined === input.limit || "number" === typeof input.limit) &&
          (undefined === input.startDay ||
            ("number" === typeof input.startDay &&
              __typia_transform__isTypeInt32._isTypeInt32(input.startDay))) &&
          (undefined === input.endDay ||
            ("number" === typeof input.endDay &&
              __typia_transform__isTypeInt32._isTypeInt32(input.endDay)))
        );
      };
      var _ao0 = function (input, _path, _exceptionable) {
        if (_exceptionable === void 0) {
          _exceptionable = true;
        }
        return (
          (undefined === input.orderBy ||
            "totalRows" === input.orderBy ||
            "rowsWithText" === input.orderBy ||
            "maxTextLength" === input.orderBy ||
            "totalTextLength" === input.orderBy ||
            __typia_transform__assertGuard._assertGuard(
              _exceptionable,
              {
                method: "____moose____typia.http.createAssertQuery",
                path: _path + ".orderBy",
                expected:
                  '("maxTextLength" | "rowsWithText" | "totalRows" | "totalTextLength" | undefined)',
                value: input.orderBy,
              },
              _errorFactory,
            )) &&
          (undefined === input.limit ||
            "number" === typeof input.limit ||
            __typia_transform__assertGuard._assertGuard(
              _exceptionable,
              {
                method: "____moose____typia.http.createAssertQuery",
                path: _path + ".limit",
                expected: "(number | undefined)",
                value: input.limit,
              },
              _errorFactory,
            )) &&
          (undefined === input.startDay ||
            ("number" === typeof input.startDay &&
              (__typia_transform__isTypeInt32._isTypeInt32(input.startDay) ||
                __typia_transform__assertGuard._assertGuard(
                  _exceptionable,
                  {
                    method: "____moose____typia.http.createAssertQuery",
                    path: _path + ".startDay",
                    expected: 'number & Type<"int32">',
                    value: input.startDay,
                  },
                  _errorFactory,
                ))) ||
            __typia_transform__assertGuard._assertGuard(
              _exceptionable,
              {
                method: "____moose____typia.http.createAssertQuery",
                path: _path + ".startDay",
                expected: '((number & Type<"int32">) | undefined)',
                value: input.startDay,
              },
              _errorFactory,
            )) &&
          (undefined === input.endDay ||
            ("number" === typeof input.endDay &&
              (__typia_transform__isTypeInt32._isTypeInt32(input.endDay) ||
                __typia_transform__assertGuard._assertGuard(
                  _exceptionable,
                  {
                    method: "____moose____typia.http.createAssertQuery",
                    path: _path + ".endDay",
                    expected: 'number & Type<"int32">',
                    value: input.endDay,
                  },
                  _errorFactory,
                ))) ||
            __typia_transform__assertGuard._assertGuard(
              _exceptionable,
              {
                method: "____moose____typia.http.createAssertQuery",
                path: _path + ".endDay",
                expected: '((number & Type<"int32">) | undefined)',
                value: input.endDay,
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
        var _a, _b, _c, _d;
        input =
          __typia_transform__httpQueryParseURLSearchParams._httpQueryParseURLSearchParams(
            input,
          );
        var output = {
          orderBy:
            (_a = __typia_transform__httpQueryReadString._httpQueryReadString(
              input.get("orderBy"),
            )) !== null && _a !== void 0
              ? _a
              : undefined,
          limit:
            (_b = __typia_transform__httpQueryReadNumber._httpQueryReadNumber(
              input.get("limit"),
            )) !== null && _b !== void 0
              ? _b
              : undefined,
          startDay:
            (_c = __typia_transform__httpQueryReadNumber._httpQueryReadNumber(
              input.get("startDay"),
            )) !== null && _c !== void 0
              ? _c
              : undefined,
          endDay:
            (_d = __typia_transform__httpQueryReadNumber._httpQueryReadNumber(
              input.get("endDay"),
            )) !== null && _d !== void 0
              ? _d
              : undefined,
        };
        return output;
      };
      return function (input, errorFactory) {
        return __assert(__decode(input), errorFactory);
      };
    })();
    var searchParams = new URLSearchParams(params);
    var processedParams = assertGuard(searchParams);
    return (function (_a, _b) {
      return __awaiter(void 0, [_a, _b], void 0, function (_c, _d) {
        var BACols, query, data;
        var _e = _c.orderBy,
          orderBy = _e === void 0 ? "totalRows" : _e,
          _f = _c.limit,
          limit = _f === void 0 ? 5 : _f,
          _g = _c.startDay,
          startDay = _g === void 0 ? 1 : _g,
          _h = _c.endDay,
          endDay = _h === void 0 ? 31 : _h;
        var client = _d.client,
          sql = _d.sql;
        return __generator(this, function (_j) {
          switch (_j.label) {
            case 0:
              BACols = views_1.BarAggregatedMV.targetTable.columns;
              query = sql(
                templateObject_1 ||
                  (templateObject_1 = __makeTemplateObject(
                    [
                      "\n        SELECT \n          ",
                      " as dayOfMonth,\n          ",
                      "\n        FROM ",
                      "\n        WHERE \n          dayOfMonth >= ",
                      " \n          AND dayOfMonth <= ",
                      "\n        ORDER BY ",
                      " DESC\n        LIMIT ",
                      "\n      ",
                    ],
                    [
                      "\n        SELECT \n          ",
                      " as dayOfMonth,\n          ",
                      "\n        FROM ",
                      "\n        WHERE \n          dayOfMonth >= ",
                      " \n          AND dayOfMonth <= ",
                      "\n        ORDER BY ",
                      " DESC\n        LIMIT ",
                      "\n      ",
                    ],
                  )),
                BACols.dayOfMonth,
                BACols[orderBy],
                views_1.BarAggregatedMV.targetTable.name,
                startDay,
                endDay,
                BACols[orderBy],
                limit,
              );
              return [4 /*yield*/, client.query.execute(query)];
            case 1:
              data = _j.sent();
              return [4 /*yield*/, data.json()];
            case 2:
              return [2 /*return*/, _j.sent()];
          }
        });
      });
    })(processedParams, utils);
  },
  {},
  {
    version: "3.1",
    components: {
      schemas: {
        QueryParams: {
          type: "object",
          properties: {
            orderBy: {
              oneOf: [
                {
                  const: "totalRows",
                },
                {
                  const: "rowsWithText",
                },
                {
                  const: "maxTextLength",
                },
                {
                  const: "totalTextLength",
                },
              ],
            },
            limit: {
              type: "number",
            },
            startDay: {
              type: "integer",
            },
            endDay: {
              type: "integer",
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
  },
  JSON.parse(
    '[{"name":"orderBy","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"limit","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"startDay","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"endDay","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]}]',
  ),
  {
    version: "3.1",
    components: {
      schemas: {
        ResponseBody: {
          type: "object",
          properties: {
            dayOfMonth: {
              type: "number",
            },
            totalRows: {
              type: "number",
            },
            rowsWithText: {
              type: "number",
            },
            maxTextLength: {
              type: "number",
            },
            totalTextLength: {
              type: "number",
            },
          },
          required: ["dayOfMonth"],
        },
      },
    },
    schemas: [
      {
        type: "array",
        items: {
          $ref: "#/components/schemas/ResponseBody",
        },
      },
    ],
  },
);
var templateObject_1;
//# sourceMappingURL=bar.js.map
