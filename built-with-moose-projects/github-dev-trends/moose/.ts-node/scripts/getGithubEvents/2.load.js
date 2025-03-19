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
var load = function (input) {
  return __awaiter(void 0, void 0, void 0, function () {
    var octokit,
      _i,
      _a,
      event_1,
      mooseEvent,
      repo,
      repoData,
      mooseEventWithRepo;
    var _b, _c, _d;
    return __generator(this, function (_e) {
      switch (_e.label) {
        case 0:
          // The body of your script goes here
          console.log("Hello world from load");
          if (!input || input.noNewEvents) {
            console.log("No new events to load");
            return [
              2 /*return*/,
              {
                task: "load",
                data: {
                  eventsLoaded: 0,
                },
              },
            ];
          }
          console.log(
            "Loading "
              .concat(input.count, " events from ")
              .concat(input.fetchedAt),
          );
          octokit = (0, utils_1.createOctokit)();
          (_i = 0), (_a = input.events);
          _e.label = 1;
        case 1:
          if (!(_i < _a.length)) return [3 /*break*/, 5];
          event_1 = _a[_i];
          if (!(event_1.type === "WatchEvent")) return [3 /*break*/, 4];
          mooseEvent = {
            eventId: event_1.id,
            actorLogin: event_1.actor.login,
            actorId: event_1.actor.id,
            actorUrl: event_1.actor.url,
            actorAvatarUrl: event_1.actor.avatar_url,
            repoName: event_1.repo.name,
            repoUrl: event_1.repo.url,
            repoId: event_1.repo.id,
            createdAt: event_1.created_at ? new Date(event_1.created_at) : null,
          };
          return [
            4 /*yield*/,
            octokit.rest.repos.get({
              owner: event_1.repo.name.split("/")[0],
              repo: event_1.repo.name.split("/")[1],
            }),
          ];
        case 2:
          repo = _e.sent();
          repoData = repo.data;
          mooseEventWithRepo = __assign(__assign({}, mooseEvent), {
            repoDescription: repoData.description,
            repoTopics: repoData.topics,
            repoLanguage: repoData.language,
            repoStars: repoData.stargazers_count,
            repoForks: repoData.forks_count,
            repoWatchers: repoData.watchers_count,
            repoOpenIssues: repoData.open_issues_count,
            repoCreatedAt: repoData.created_at
              ? new Date(repoData.created_at)
              : null,
            repoOwnerLogin: repoData.owner.login,
            repoOwnerId: repoData.owner.id,
            repoOwnerUrl: repoData.owner.url,
            repoOwnerAvatarUrl: repoData.owner.avatar_url,
            repoOwnerType: repoData.owner.type,
            repoOrgId:
              (_b = repoData.organization) === null || _b === void 0
                ? void 0
                : _b.id,
            repoOrgUrl:
              (_c = repoData.organization) === null || _c === void 0
                ? void 0
                : _c.url,
            repoOrgLogin:
              (_d = repoData.organization) === null || _d === void 0
                ? void 0
                : _d.login,
            repoHomepage: repoData.homepage,
          });
          return [
            4 /*yield*/,
            fetch("http://localhost:4000/ingest/WatchEventWithRepo", {
              method: "POST",
              body: JSON.stringify(mooseEventWithRepo),
            }),
          ];
        case 3:
          _e.sent();
          (0, moose_lib_1.cliLog)({
            action: "load",
            message: JSON.stringify(mooseEventWithRepo),
          });
          _e.label = 4;
        case 4:
          _i++;
          return [3 /*break*/, 1];
        case 5:
          // The return value is the output of the script.
          // The return value should be a dictionary with at least:
          // - task: the task name (e.g., "extract", "transform")
          // - data: the actual data being passed to the next task
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
