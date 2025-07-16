"use strict";
var __createBinding =
  (this && this.__createBinding) ||
  (Object.create ?
    function (o, m, k, k2) {
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
  (Object.create ?
    function (o, v) {
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
      return value instanceof P ? value : (
          new P(function (resolve) {
            resolve(value);
          })
        );
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
        result.done ?
          resolve(result.value)
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
                op[0] & 2 ? y["return"]
                : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
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
exports.workflow = exports.ingest = void 0;
var __typia_transform__validateReport = __importStar(
  require("typia/lib/internal/_validateReport.js"),
);
var __typia_transform__assertGuard = __importStar(
  require("typia/lib/internal/_assertGuard.js"),
);
var typia_1 = __importDefault(require("typia"));
var moose_lib_1 = require("@514labs/moose-lib");
var faker_1 = require("@faker-js/faker");
// Create OLAP Table
var workflowTable = new moose_lib_1.OlapTable(
  "FooWorkflow",
  {},
  {
    version: "3.1",
    components: {
      schemas: {
        FooWorkflow: {
          type: "object",
          properties: {
            id: {
              type: "string",
            },
            success: {
              type: "boolean",
            },
            message: {
              type: "string",
            },
          },
          required: ["id", "success", "message"],
        },
      },
    },
    schemas: [
      {
        $ref: "#/components/schemas/FooWorkflow",
      },
    ],
  },
  JSON.parse(
    '[{"name":"id","data_type":"String","primary_key":true,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"success","data_type":"Boolean","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"message","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]}]',
  ),
  {
    validate: function (data) {
      var result = (function () {
        var _io0 = function (input) {
          return (
            "string" === typeof input.id &&
            "boolean" === typeof input.success &&
            "string" === typeof input.message
          );
        };
        var _vo0 = function (input, _path, _exceptionable) {
          if (_exceptionable === void 0) {
            _exceptionable = true;
          }
          return [
            "string" === typeof input.id ||
              _report(_exceptionable, {
                path: _path + ".id",
                expected: "string",
                value: input.id,
              }),
            "boolean" === typeof input.success ||
              _report(_exceptionable, {
                path: _path + ".success",
                expected: "boolean",
                value: input.success,
              }),
            "string" === typeof input.message ||
              _report(_exceptionable, {
                path: _path + ".message",
                expected: "string",
                value: input.message,
              }),
          ].every(function (flag) {
            return flag;
          });
        };
        var __is = function (input) {
          return "object" === typeof input && null !== input && _io0(input);
        };
        var errors;
        var _report;
        return function (input) {
          if (false === __is(input)) {
            errors = [];
            _report = __typia_transform__validateReport._validateReport(errors);
            (function (input, _path, _exceptionable) {
              if (_exceptionable === void 0) {
                _exceptionable = true;
              }
              return (
                ((("object" === typeof input && null !== input) ||
                  _report(true, {
                    path: _path + "",
                    expected: "FooWorkflow",
                    value: input,
                  })) &&
                  _vo0(input, _path + "", true)) ||
                _report(true, {
                  path: _path + "",
                  expected: "FooWorkflow",
                  value: input,
                })
              );
            })(input, "$input", true);
            var success = 0 === errors.length;
            return success ?
                {
                  success: success,
                  data: input,
                }
              : {
                  success: success,
                  errors: errors,
                  data: input,
                };
          }
          return {
            success: true,
            data: input,
          };
        };
      })()(data);
      return {
        success: result.success,
        data: result.success ? result.data : undefined,
        errors: result.success ? undefined : result.errors,
      };
    },
    assert: (function () {
      var _io0 = function (input) {
        return (
          "string" === typeof input.id &&
          "boolean" === typeof input.success &&
          "string" === typeof input.message
        );
      };
      var _ao0 = function (input, _path, _exceptionable) {
        if (_exceptionable === void 0) {
          _exceptionable = true;
        }
        return (
          ("string" === typeof input.id ||
            __typia_transform__assertGuard._assertGuard(
              _exceptionable,
              {
                method: "____moose____typia.createAssert",
                path: _path + ".id",
                expected: "string",
                value: input.id,
              },
              _errorFactory,
            )) &&
          ("boolean" === typeof input.success ||
            __typia_transform__assertGuard._assertGuard(
              _exceptionable,
              {
                method: "____moose____typia.createAssert",
                path: _path + ".success",
                expected: "boolean",
                value: input.success,
              },
              _errorFactory,
            )) &&
          ("string" === typeof input.message ||
            __typia_transform__assertGuard._assertGuard(
              _exceptionable,
              {
                method: "____moose____typia.createAssert",
                path: _path + ".message",
                expected: "string",
                value: input.message,
              },
              _errorFactory,
            ))
        );
      };
      var __is = function (input) {
        return "object" === typeof input && null !== input && _io0(input);
      };
      var _errorFactory;
      return function (input, errorFactory) {
        if (false === __is(input)) {
          _errorFactory = errorFactory;
          (function (input, _path, _exceptionable) {
            if (_exceptionable === void 0) {
              _exceptionable = true;
            }
            return (
              ((("object" === typeof input && null !== input) ||
                __typia_transform__assertGuard._assertGuard(
                  true,
                  {
                    method: "____moose____typia.createAssert",
                    path: _path + "",
                    expected: "FooWorkflow",
                    value: input,
                  },
                  _errorFactory,
                )) &&
                _ao0(input, _path + "", true)) ||
              __typia_transform__assertGuard._assertGuard(
                true,
                {
                  method: "____moose____typia.createAssert",
                  path: _path + "",
                  expected: "FooWorkflow",
                  value: input,
                },
                _errorFactory,
              )
            );
          })(input, "$input", true);
        }
        return input;
      };
    })(),
    is: (function () {
      var _io0 = function (input) {
        return (
          "string" === typeof input.id &&
          "boolean" === typeof input.success &&
          "string" === typeof input.message
        );
      };
      return function (input) {
        return "object" === typeof input && null !== input && _io0(input);
      };
    })(),
  },
);
exports.ingest = new moose_lib_1.Task("ingest", {
  run: function () {
    return __awaiter(void 0, void 0, void 0, function () {
      var i, foo, response, error_1;
      return __generator(this, function (_a) {
        switch (_a.label) {
          case 0:
            i = 0;
            _a.label = 1;
          case 1:
            if (!(i < 1000)) return [3 /*break*/, 8];
            foo = {
              primaryKey: faker_1.faker.string.uuid(),
              timestamp: faker_1.faker.date.recent({ days: 365 }).getTime(),
              optionalText:
                Math.random() < 0.5 ? faker_1.faker.lorem.text() : undefined,
            };
            _a.label = 2;
          case 2:
            _a.trys.push([2, 4, , 5]);
            return [
              4 /*yield*/,
              fetch("http://localhost:4000/ingest/Foo", {
                method: "POST",
                headers: {
                  "Content-Type": "application/json",
                },
                body: JSON.stringify(foo),
              }),
            ];
          case 3:
            response = _a.sent();
            if (!response.ok) {
              console.log(
                "Failed to ingest record "
                  .concat(i, ": ")
                  .concat(response.status, " ")
                  .concat(response.statusText),
              );
              // Insert ingestion result into OLAP table
              workflowTable.insert([
                { id: "1", success: false, message: response.statusText },
              ]);
            }
            return [3 /*break*/, 5];
          case 4:
            error_1 = _a.sent();
            console.log(
              "Error ingesting record ".concat(i, ": ").concat(error_1),
            );
            workflowTable.insert([
              { id: "1", success: false, message: error_1.message },
            ]);
            return [3 /*break*/, 5];
          case 5:
            if (!(i % 100 === 0)) return [3 /*break*/, 7];
            console.log("Ingested ".concat(i, " records..."));
            workflowTable.insert([
              {
                id: "1",
                success: true,
                message: "Ingested ".concat(i, " records"),
              },
            ]);
            return [
              4 /*yield*/,
              new Promise(function (resolve) {
                return setTimeout(resolve, 100);
              }),
            ];
          case 6:
            _a.sent();
            _a.label = 7;
          case 7:
            i++;
            return [3 /*break*/, 1];
          case 8:
            return [2 /*return*/];
        }
      });
    });
  },
  retries: 3,
  timeout: "30s",
});
exports.workflow = new moose_lib_1.Workflow("workflow", {
  startingTask: exports.ingest,
  retries: 3,
  timeout: "30s",
  schedule: "@every 5s",
});
//# sourceMappingURL=generateRandom.js.map
