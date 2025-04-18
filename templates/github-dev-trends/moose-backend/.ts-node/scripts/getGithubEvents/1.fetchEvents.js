"use strict";
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
var __asyncValues =
  (this && this.__asyncValues) ||
  function (o) {
    if (!Symbol.asyncIterator)
      throw new TypeError("Symbol.asyncIterator is not defined.");
    var m = o[Symbol.asyncIterator],
      i;
    return m
      ? m.call(o)
      : ((o =
          typeof __values === "function" ? __values(o) : o[Symbol.iterator]()),
        (i = {}),
        verb("next"),
        verb("throw"),
        verb("return"),
        (i[Symbol.asyncIterator] = function () {
          return this;
        }),
        i);
    function verb(n) {
      i[n] =
        o[n] &&
        function (v) {
          return new Promise(function (resolve, reject) {
            (v = o[n](v)), settle(resolve, reject, v.done, v.value);
          });
        };
    }
    function settle(resolve, reject, d, v) {
      Promise.resolve(v).then(function (v) {
        resolve({ value: v, done: d });
      }, reject);
    }
  };
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = createTask;
var utils_1 = require("../../utils");
var octokit = (0, utils_1.createOctokit)();
// The initial input data and data passed between tasks can be
// defined in the task function parameter
var load = function (input) {
  return __awaiter(void 0, void 0, void 0, function () {
    var responses,
      _a,
      responses_1,
      responses_1_1,
      response,
      _i,
      _b,
      event_1,
      ghEvent,
      e_1_1;
    var _c, e_1, _d, _e;
    return __generator(this, function (_f) {
      switch (_f.label) {
        case 0:
          return [
            4 /*yield*/,
            octokit.paginate.iterator(octokit.activity.listPublicEvents, {
              per_page: 100,
            }),
          ];
        case 1:
          responses = _f.sent();
          _f.label = 2;
        case 2:
          _f.trys.push([2, 10, 11, 16]);
          (_a = true), (responses_1 = __asyncValues(responses));
          _f.label = 3;
        case 3:
          return [4 /*yield*/, responses_1.next()];
        case 4:
          if (!((responses_1_1 = _f.sent()), (_c = responses_1_1.done), !_c))
            return [3 /*break*/, 9];
          _e = responses_1_1.value;
          _a = false;
          response = _e;
          (_i = 0), (_b = response.data);
          _f.label = 5;
        case 5:
          if (!(_i < _b.length)) return [3 /*break*/, 8];
          event_1 = _b[_i];
          ghEvent = {
            eventType: event_1.type,
            eventId: event_1.id,
            actorLogin: event_1.actor.login,
            actorId: event_1.actor.id,
            actorUrl: event_1.actor.url,
            actorAvatarUrl: event_1.actor.avatar_url,
            repoFullName: event_1.repo.name,
            repoOwner: event_1.repo.name.split("/")[0],
            repoName: event_1.repo.name.split("/")[1],
            repoUrl: event_1.repo.url,
            repoId: event_1.repo.id,
            createdAt: event_1.created_at
              ? new Date(event_1.created_at)
              : new Date(),
          };
          return [
            4 /*yield*/,
            fetch("http://localhost:4000/ingest/GhEvent", {
              method: "POST",
              body: JSON.stringify(ghEvent),
            }),
          ];
        case 6:
          _f.sent();
          _f.label = 7;
        case 7:
          _i++;
          return [3 /*break*/, 5];
        case 8:
          _a = true;
          return [3 /*break*/, 3];
        case 9:
          return [3 /*break*/, 16];
        case 10:
          e_1_1 = _f.sent();
          e_1 = { error: e_1_1 };
          return [3 /*break*/, 16];
        case 11:
          _f.trys.push([11, , 14, 15]);
          if (!(!_a && !_c && (_d = responses_1.return)))
            return [3 /*break*/, 13];
          return [4 /*yield*/, _d.call(responses_1)];
        case 12:
          _f.sent();
          _f.label = 13;
        case 13:
          return [3 /*break*/, 15];
        case 14:
          if (e_1) throw e_1.error;
          return [7 /*endfinally*/];
        case 15:
          return [7 /*endfinally*/];
        case 16:
          return [
            2 /*return*/,
            {
              task: "load",
              data: {},
            },
          ];
      }
    });
  });
};
function createTask() {
  return {
    task: load,
    config: {
      retries: 3,
    },
  };
}
//# sourceMappingURL=1.fetchEvents.js.map
