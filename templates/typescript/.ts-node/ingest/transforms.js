"use strict";
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
Object.defineProperty(exports, "__esModule", { value: true });
var models_1 = require("./models");
var moose_lib_1 = require("@514labs/moose-lib");
// Transform Foo events to Bar events
models_1.FooPipeline.stream.addTransform(
  models_1.BarPipeline.stream,
  function (foo) {
    return __awaiter(void 0, void 0, void 0, function () {
      var cache, cacheKey, cached, result;
      var _a, _b;
      return __generator(this, function (_c) {
        switch (_c.label) {
          case 0:
            return [4 /*yield*/, moose_lib_1.MooseCache.get()];
          case 1:
            cache = _c.sent();
            cacheKey = "processed:".concat(foo.primaryKey);
            return [4 /*yield*/, cache.get(cacheKey)];
          case 2:
            cached = _c.sent();
            if (cached) {
              console.log("Using cached result for ".concat(foo.primaryKey));
              return [2 /*return*/, cached];
            }
            if (foo.timestamp === 1728000000.0) {
              // magic value to test the dead letter queue
              throw new Error("blah");
            }
            result = {
              primaryKey: foo.primaryKey,
              utcTimestamp: new Date(foo.timestamp * 1000), // Convert timestamp to Date
              hasText: foo.optionalText !== undefined,
              textLength:
                (
                  (_b =
                    (_a = foo.optionalText) === null || _a === void 0 ?
                      void 0
                    : _a.length) !== null && _b !== void 0
                ) ?
                  _b
                : 0,
            };
            // Cache the result (1 hour retention)
            return [4 /*yield*/, cache.set(cacheKey, result, 3600)];
          case 3:
            // Cache the result (1 hour retention)
            _c.sent();
            return [2 /*return*/, result];
        }
      });
    });
  },
  {
    deadLetterQueue: models_1.FooPipeline.deadLetterQueue,
  },
);
// Add a streaming consumer to print Foo events
var printFooEvent = function (foo) {
  var _a;
  console.log("Received Foo event:");
  console.log("  Primary Key: ".concat(foo.primaryKey));
  console.log("  Timestamp: ".concat(new Date(foo.timestamp * 1000)));
  console.log(
    "  Optional Text: ".concat(
      (_a = foo.optionalText) !== null && _a !== void 0 ? _a : "None",
    ),
  );
  console.log("---");
};
models_1.FooPipeline.stream.addConsumer(printFooEvent);
// DLQ consumer for handling failed events (alternate flow)
models_1.FooPipeline.deadLetterQueue.addConsumer(function (deadLetter) {
  console.log(deadLetter);
  var foo = deadLetter.asTyped();
  console.log(foo);
});
//# sourceMappingURL=transforms.js.map
