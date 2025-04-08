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
exports.getAircraftPositions = void 0;
var __typia_transform__isTypeInt64 = __importStar(
  require("typia/lib/internal/_isTypeInt64.js"),
);
var __typia_transform__assertGuard = __importStar(
  require("typia/lib/internal/_assertGuard.js"),
);
var __typia_transform__httpQueryParseURLSearchParams = __importStar(
  require("typia/lib/internal/_httpQueryParseURLSearchParams.js"),
);
var __typia_transform__httpQueryReadNumber = __importStar(
  require("typia/lib/internal/_httpQueryReadNumber.js"),
);
var typia_1 = __importDefault(require("typia"));
var moose_lib_1 = require("@514labs/moose-lib");
/**
 * API to retrieve current positions and details of all active aircraft
 * Returns aircraft locations, altitude, speed, and identifying information
 */
exports.getAircraftPositions = new moose_lib_1.ConsumptionApi(
  "getAircraftPositions",
  function (params, utils) {
    var assertGuard = (function () {
      var _io0 = function (input) {
        return (
          (undefined === input.limit ||
            ("number" === typeof input.limit &&
              __typia_transform__isTypeInt64._isTypeInt64(input.limit) &&
              1 <= input.limit &&
              input.limit <= 1000)) &&
          (undefined === input.minAltitude ||
            ("number" === typeof input.minAltitude &&
              __typia_transform__isTypeInt64._isTypeInt64(input.minAltitude) &&
              0 <= input.minAltitude)) &&
          (undefined === input.maxAltitude ||
            ("number" === typeof input.maxAltitude &&
              __typia_transform__isTypeInt64._isTypeInt64(input.maxAltitude) &&
              0 <= input.maxAltitude))
        );
      };
      var _ao0 = function (input, _path, _exceptionable) {
        if (_exceptionable === void 0) {
          _exceptionable = true;
        }
        return (
          (undefined === input.limit ||
            ("number" === typeof input.limit &&
              (__typia_transform__isTypeInt64._isTypeInt64(input.limit) ||
                __typia_transform__assertGuard._assertGuard(
                  _exceptionable,
                  {
                    method: "____moose____typia.http.createAssertQuery",
                    path: _path + ".limit",
                    expected: 'number & Type<"int64">',
                    value: input.limit,
                  },
                  _errorFactory,
                )) &&
              (1 <= input.limit ||
                __typia_transform__assertGuard._assertGuard(
                  _exceptionable,
                  {
                    method: "____moose____typia.http.createAssertQuery",
                    path: _path + ".limit",
                    expected: "number & Minimum<1>",
                    value: input.limit,
                  },
                  _errorFactory,
                )) &&
              (input.limit <= 1000 ||
                __typia_transform__assertGuard._assertGuard(
                  _exceptionable,
                  {
                    method: "____moose____typia.http.createAssertQuery",
                    path: _path + ".limit",
                    expected: "number & Maximum<1000>",
                    value: input.limit,
                  },
                  _errorFactory,
                ))) ||
            __typia_transform__assertGuard._assertGuard(
              _exceptionable,
              {
                method: "____moose____typia.http.createAssertQuery",
                path: _path + ".limit",
                expected:
                  '((number & Type<"int64"> & Minimum<1> & Maximum<1000>) | undefined)',
                value: input.limit,
              },
              _errorFactory,
            )) &&
          (undefined === input.minAltitude ||
            ("number" === typeof input.minAltitude &&
              (__typia_transform__isTypeInt64._isTypeInt64(input.minAltitude) ||
                __typia_transform__assertGuard._assertGuard(
                  _exceptionable,
                  {
                    method: "____moose____typia.http.createAssertQuery",
                    path: _path + ".minAltitude",
                    expected: 'number & Type<"int64">',
                    value: input.minAltitude,
                  },
                  _errorFactory,
                )) &&
              (0 <= input.minAltitude ||
                __typia_transform__assertGuard._assertGuard(
                  _exceptionable,
                  {
                    method: "____moose____typia.http.createAssertQuery",
                    path: _path + ".minAltitude",
                    expected: "number & Minimum<0>",
                    value: input.minAltitude,
                  },
                  _errorFactory,
                ))) ||
            __typia_transform__assertGuard._assertGuard(
              _exceptionable,
              {
                method: "____moose____typia.http.createAssertQuery",
                path: _path + ".minAltitude",
                expected: '((number & Type<"int64"> & Minimum<0>) | undefined)',
                value: input.minAltitude,
              },
              _errorFactory,
            )) &&
          (undefined === input.maxAltitude ||
            ("number" === typeof input.maxAltitude &&
              (__typia_transform__isTypeInt64._isTypeInt64(input.maxAltitude) ||
                __typia_transform__assertGuard._assertGuard(
                  _exceptionable,
                  {
                    method: "____moose____typia.http.createAssertQuery",
                    path: _path + ".maxAltitude",
                    expected: 'number & Type<"int64">',
                    value: input.maxAltitude,
                  },
                  _errorFactory,
                )) &&
              (0 <= input.maxAltitude ||
                __typia_transform__assertGuard._assertGuard(
                  _exceptionable,
                  {
                    method: "____moose____typia.http.createAssertQuery",
                    path: _path + ".maxAltitude",
                    expected: "number & Minimum<0>",
                    value: input.maxAltitude,
                  },
                  _errorFactory,
                ))) ||
            __typia_transform__assertGuard._assertGuard(
              _exceptionable,
              {
                method: "____moose____typia.http.createAssertQuery",
                path: _path + ".maxAltitude",
                expected: '((number & Type<"int64"> & Minimum<0>) | undefined)',
                value: input.maxAltitude,
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
                    expected: "AircraftPositionsParams",
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
                  expected: "AircraftPositionsParams",
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
          limit:
            (_a = __typia_transform__httpQueryReadNumber._httpQueryReadNumber(
              input.get("limit"),
            )) !== null && _a !== void 0
              ? _a
              : undefined,
          minAltitude:
            (_b = __typia_transform__httpQueryReadNumber._httpQueryReadNumber(
              input.get("minAltitude"),
            )) !== null && _b !== void 0
              ? _b
              : undefined,
          maxAltitude:
            (_c = __typia_transform__httpQueryReadNumber._httpQueryReadNumber(
              input.get("maxAltitude"),
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
    var searchParams = new URLSearchParams(params);
    var processedParams = assertGuard(searchParams);
    return (function (params, utils) {
      return __awaiter(void 0, void 0, void 0, function () {
        var client, sql, limit, query;
        var _a;
        return __generator(this, function (_b) {
          switch (_b.label) {
            case 0:
              (client = utils.client), (sql = utils.sql);
              limit = (_a = params.limit) !== null && _a !== void 0 ? _a : 100;
              query = sql(
                templateObject_1 ||
                  (templateObject_1 = __makeTemplateObject(
                    [
                      "\n      SELECT \n        hex,\n        flight,\n        aircraft_type,\n        category,\n        lat,\n        lon,\n        alt_baro as altitude,\n        gs as ground_speed,\n        track as heading,\n        emergency,\n        alt_baro_is_ground as is_on_ground,\n        timestamp\n      FROM AircraftTrackingProcessed\n      WHERE lat != 0 AND lon != 0  -- Filter out invalid positions\n        AND timestamp >= now() - INTERVAL 30 SECOND  -- Only show recent aircraft\n    ",
                    ],
                    [
                      "\n      SELECT \n        hex,\n        flight,\n        aircraft_type,\n        category,\n        lat,\n        lon,\n        alt_baro as altitude,\n        gs as ground_speed,\n        track as heading,\n        emergency,\n        alt_baro_is_ground as is_on_ground,\n        timestamp\n      FROM AircraftTrackingProcessed\n      WHERE lat != 0 AND lon != 0  -- Filter out invalid positions\n        AND timestamp >= now() - INTERVAL 30 SECOND  -- Only show recent aircraft\n    ",
                    ],
                  )),
              );
              // Add minimum altitude filter if provided
              if (params.minAltitude !== undefined) {
                query = sql(
                  templateObject_2 ||
                    (templateObject_2 = __makeTemplateObject(
                      ["", " AND alt_baro >= ", ""],
                      ["", " AND alt_baro >= ", ""],
                    )),
                  query,
                  params.minAltitude,
                );
              }
              // Add maximum altitude filter if provided
              if (params.maxAltitude !== undefined) {
                query = sql(
                  templateObject_3 ||
                    (templateObject_3 = __makeTemplateObject(
                      ["", " AND alt_baro <= ", ""],
                      ["", " AND alt_baro <= ", ""],
                    )),
                  query,
                  params.maxAltitude,
                );
              }
              // Add ordering and limit
              query = sql(
                templateObject_4 ||
                  (templateObject_4 = __makeTemplateObject(
                    [
                      "\n      ",
                      "\n      ORDER BY timestamp DESC\n      LIMIT ",
                      "\n    ",
                    ],
                    [
                      "\n      ",
                      "\n      ORDER BY timestamp DESC\n      LIMIT ",
                      "\n    ",
                    ],
                  )),
                query,
                limit,
              );
              return [4 /*yield*/, client.query.execute(query)];
            case 1:
              // Execute the query and return results
              return [2 /*return*/, _b.sent()];
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
        AircraftPositionsParams: {
          type: "object",
          properties: {
            limit: {
              type: "integer",
              minimum: 1,
              maximum: 1000,
              description: "Maximum number of aircraft to return (1-1000)",
            },
            minAltitude: {
              type: "integer",
              minimum: 0,
              description: "Minimum altitude in feet to filter by (optional)",
            },
            maxAltitude: {
              type: "integer",
              minimum: 0,
              description: "Maximum altitude in feet to filter by (optional)",
            },
          },
          required: [],
          description: "Parameters for the aircraft positions API",
        },
      },
    },
    schemas: [
      {
        $ref: "#/components/schemas/AircraftPositionsParams",
      },
    ],
  },
  JSON.parse(
    '[{"name":"limit","data_type":"Int","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"minAltitude","data_type":"Int","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"maxAltitude","data_type":"Int","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]}]',
  ),
  {
    version: "3.1",
    components: {
      schemas: {},
    },
    schemas: [{}],
  },
);
var templateObject_1, templateObject_2, templateObject_3, templateObject_4;
//# sourceMappingURL=getAircraftPositions.js.map
