"use strict";
var __assign =
  (this && this.__assign) ||
  function () {
    __assign =
      Object.assign ||
      function (t) {
        for (var s, i = 1, n = arguments.length; i < n; i++) {
          s = arguments[i];
          for (var p in s)
            if (Object.prototype.hasOwnProperty.call(s, p)) t[p] = s[p];
        }
        return t;
      };
    return __assign.apply(this, arguments);
  };
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
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = createTask;
var utils_1 = require("../utils");
var moose_lib_1 = require("@514labs/moose-lib");
// The initial input data and data passed between tasks can be
// defined in the task function parameter
var EVENT_TYPES = new Set([
  "WatchEvent",
  "PushEvent",
  "ForkEvent",
  "PullRequestEvent",
  "CreatedEvent",
]);
var BATCH_SIZE = 5;
var DELAY_BETWEEN_BATCHES = 1000;
var load = function (input) {
  return __awaiter(void 0, void 0, void 0, function () {
    var repos,
      repoDataMap,
      i,
      batch,
      batchResults,
      error_1,
      _i,
      _a,
      event_1,
      mooseEvent,
      repoData,
      mooseEventWithRepo;
    var _b, _c, _d, _e;
    return __generator(this, function (_f) {
      switch (_f.label) {
        case 0:
          if (!input || input.noNewEvents) {
            return [
              2 /*return*/,
              {
                task: "load",
                data: { eventsLoaded: 0 },
              },
            ];
          }
          repos = Array.from(
            new Set(
              input.events
                .filter(function (event) {
                  return event.type && EVENT_TYPES.has(event.type);
                })
                .map(function (event) {
                  return event.repo.name;
                }),
            ),
          ).map(function (fullName) {
            var _a = fullName.split("/"),
              owner = _a[0],
              name = _a[1];
            return { owner: owner, name: name };
          });
          repoDataMap = new Map();
          i = 0;
          _f.label = 1;
        case 1:
          if (!(i < repos.length)) return [3 /*break*/, 8];
          if (!(i > 0)) return [3 /*break*/, 3];
          return [
            4 /*yield*/,
            new Promise(function (resolve) {
              return setTimeout(resolve, DELAY_BETWEEN_BATCHES);
            }),
          ];
        case 2:
          _f.sent();
          _f.label = 3;
        case 3:
          batch = repos.slice(i, i + BATCH_SIZE);
          _f.label = 4;
        case 4:
          _f.trys.push([4, 6, , 7]);
          return [4 /*yield*/, (0, utils_1.fetchReposBatchWithRetry)(batch)];
        case 5:
          batchResults = _f.sent();
          batchResults.forEach(function (value, key) {
            return repoDataMap.set(key, value);
          });
          (0, moose_lib_1.cliLog)({
            action: "fetchRepos",
            message: "Processed batch "
              .concat(Math.floor(i / BATCH_SIZE) + 1, "/")
              .concat(Math.ceil(repos.length / BATCH_SIZE)),
            message_type: "Info",
          });
          return [3 /*break*/, 7];
        case 6:
          error_1 = _f.sent();
          (0, moose_lib_1.cliLog)({
            action: "fetchRepos",
            message: "Error fetching repos batch: ".concat(error_1),
            message_type: "Error",
          });
          return [3 /*break*/, 7];
        case 7:
          i += BATCH_SIZE;
          return [3 /*break*/, 1];
        case 8:
          (_i = 0), (_a = input.events);
          _f.label = 9;
        case 9:
          if (!(_i < _a.length)) return [3 /*break*/, 12];
          event_1 = _a[_i];
          if (!(event_1.type && EVENT_TYPES.has(event_1.type)))
            return [3 /*break*/, 11];
          mooseEvent = {
            eventId: event_1.id,
            actorLogin: event_1.actor.login,
            actorId: event_1.actor.id,
            actorUrl: event_1.actor.url,
            actorAvatarUrl: event_1.actor.avatar_url,
            repoName: event_1.repo.name,
            repoUrl: event_1.repo.url,
            repoId: event_1.repo.id,
            createdAt: event_1.created_at
              ? new Date(event_1.created_at)
              : new Date(),
          };
          repoData = repoDataMap.get(event_1.repo.name);
          if (!repoData) return [3 /*break*/, 11];
          mooseEventWithRepo = __assign(__assign({}, mooseEvent), {
            repoDescription:
              (_b = repoData.description) !== null && _b !== void 0
                ? _b
                : undefined,
            repoTopics: repoData.repositoryTopics.nodes.map(function (n) {
              return n.topic.name;
            }),
            repoLanguage:
              (_d =
                (_c = repoData.primaryLanguage) === null || _c === void 0
                  ? void 0
                  : _c.name) !== null && _d !== void 0
                ? _d
                : undefined,
            repoStars: repoData.stargazerCount,
            repoForks: repoData.forkCount,
            repoWatchers: repoData.watchers.totalCount,
            repoOpenIssues: repoData.issues.totalCount,
            repoCreatedAt: new Date(repoData.createdAt),
            repoOwnerLogin: repoData.owner.login,
            repoOwnerId: repoData.owner.id,
            repoOwnerUrl: repoData.owner.url,
            repoOwnerAvatarUrl: repoData.owner.avatarUrl,
            repoOwnerType: repoData.owner.__typename,
            repoOrgId:
              repoData.owner.__typename === "Organization"
                ? parseInt(repoData.owner.id)
                : undefined,
            repoOrgUrl:
              repoData.owner.__typename === "Organization"
                ? repoData.owner.url
                : undefined,
            repoOrgLogin:
              repoData.owner.__typename === "Organization"
                ? repoData.owner.login
                : undefined,
            repoHomepage:
              (_e = repoData.homepageUrl) !== null && _e !== void 0
                ? _e
                : undefined,
          });
          return [
            4 /*yield*/,
            fetch("http://localhost:4000/ingest/WatchEventWithRepo", {
              method: "POST",
              body: JSON.stringify(mooseEventWithRepo),
            }),
          ];
        case 10:
          _f.sent();
          _f.label = 11;
        case 11:
          _i++;
          return [3 /*break*/, 9];
        case 12:
          return [
            2 /*return*/,
            {
              task: "load",
              data: {
                eventsLoaded: input.count,
                loadedAt: new Date().toISOString(),
              },
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
//# sourceMappingURL=2.load.js.map
