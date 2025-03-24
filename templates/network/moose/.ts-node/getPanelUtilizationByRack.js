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
// createConsumptionApi uses compile time code generation to generate a parser for QueryParams
exports.default = (function () {
  var assertGuard = (function () {
    var _io0 = function (input) {
      return (
        (undefined === input.minUtilization ||
          ("number" === typeof input.minUtilization &&
            1 <= input.minUtilization &&
            input.minUtilization <= 100)) &&
        (undefined === input.maxUtilization ||
          ("number" === typeof input.maxUtilization &&
            1 <= input.maxUtilization &&
            input.maxUtilization <= 100))
      );
    };
    var _ao0 = function (input, _path, _exceptionable) {
      if (_exceptionable === void 0) {
        _exceptionable = true;
      }
      return (
        (undefined === input.minUtilization ||
          ("number" === typeof input.minUtilization &&
            (1 <= input.minUtilization ||
              __typia_transform__assertGuard._assertGuard(
                _exceptionable,
                {
                  method: "____moose____typia.http.createAssertQuery",
                  path: _path + ".minUtilization",
                  expected: "number & Minimum<1>",
                  value: input.minUtilization,
                },
                _errorFactory,
              )) &&
            (input.minUtilization <= 100 ||
              __typia_transform__assertGuard._assertGuard(
                _exceptionable,
                {
                  method: "____moose____typia.http.createAssertQuery",
                  path: _path + ".minUtilization",
                  expected: "number & Maximum<100>",
                  value: input.minUtilization,
                },
                _errorFactory,
              ))) ||
          __typia_transform__assertGuard._assertGuard(
            _exceptionable,
            {
              method: "____moose____typia.http.createAssertQuery",
              path: _path + ".minUtilization",
              expected: "((number & Minimum<1> & Maximum<100>) | undefined)",
              value: input.minUtilization,
            },
            _errorFactory,
          )) &&
        (undefined === input.maxUtilization ||
          ("number" === typeof input.maxUtilization &&
            (1 <= input.maxUtilization ||
              __typia_transform__assertGuard._assertGuard(
                _exceptionable,
                {
                  method: "____moose____typia.http.createAssertQuery",
                  path: _path + ".maxUtilization",
                  expected: "number & Minimum<1>",
                  value: input.maxUtilization,
                },
                _errorFactory,
              )) &&
            (input.maxUtilization <= 100 ||
              __typia_transform__assertGuard._assertGuard(
                _exceptionable,
                {
                  method: "____moose____typia.http.createAssertQuery",
                  path: _path + ".maxUtilization",
                  expected: "number & Maximum<100>",
                  value: input.maxUtilization,
                },
                _errorFactory,
              ))) ||
          __typia_transform__assertGuard._assertGuard(
            _exceptionable,
            {
              method: "____moose____typia.http.createAssertQuery",
              path: _path + ".maxUtilization",
              expected: "((number & Minimum<1> & Maximum<100>) | undefined)",
              value: input.maxUtilization,
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
      var _a, _b;
      input =
        __typia_transform__httpQueryParseURLSearchParams._httpQueryParseURLSearchParams(
          input,
        );
      var output = {
        minUtilization:
          (_a = __typia_transform__httpQueryReadNumber._httpQueryReadNumber(
            input.get("minUtilization"),
          )) !== null && _a !== void 0
            ? _a
            : undefined,
        maxUtilization:
          (_b = __typia_transform__httpQueryReadNumber._httpQueryReadNumber(
            input.get("maxUtilization"),
          )) !== null && _b !== void 0
            ? _b
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
      var query, data;
      var _e = _c.minUtilization,
        minUtilization = _e === void 0 ? 0 : _e,
        _f = _c.maxUtilization,
        maxUtilization = _f === void 0 ? 100 : _f;
      var client = _d.client,
        sql = _d.sql;
      return __generator(this, function (_g) {
        switch (_g.label) {
          case 0:
            query = sql(
              templateObject_1 ||
                (templateObject_1 = __makeTemplateObject(
                  [
                    "\n      SELECT\n        rackLocation as rack_location,\n        AVG(100.0 * connectedPorts / totalPorts) AS avg_utilization\n      FROM\n        FiberPanel_0_0\n      WHERE\n        (100.0 * connectedPorts / totalPorts) >= ",
                    "\n        AND (100.0 * connectedPorts / totalPorts) <= ",
                    "\n      GROUP BY\n        rackLocation\n      ORDER BY\n        avg_utilization DESC\n    ",
                  ],
                  [
                    "\n      SELECT\n        rackLocation as rack_location,\n        AVG(100.0 * connectedPorts / totalPorts) AS avg_utilization\n      FROM\n        FiberPanel_0_0\n      WHERE\n        (100.0 * connectedPorts / totalPorts) >= ",
                    "\n        AND (100.0 * connectedPorts / totalPorts) <= ",
                    "\n      GROUP BY\n        rackLocation\n      ORDER BY\n        avg_utilization DESC\n    ",
                  ],
                )),
              minUtilization,
              maxUtilization,
            );
            return [4 /*yield*/, client.query.execute(query)];
          case 1:
            data = _g.sent();
            return [2 /*return*/, data];
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
            minUtilization: {
              type: "number",
              minimum: 1,
              maximum: 100,
            },
            maxUtilization: {
              type: "number",
              minimum: 1,
              maximum: 100,
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
            rack_location: {
              type: "string",
            },
            avg_utilization: {
              type: "number",
            },
          },
          required: ["rack_location", "avg_utilization"],
        },
      },
    ],
  };
  return wrappedFunc;
})();
var templateObject_1;
//# sourceMappingURL=getPanelUtilizationByRack.js.map
