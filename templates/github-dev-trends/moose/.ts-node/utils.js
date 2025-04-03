"use strict";
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
Object.defineProperty(exports, "__esModule", { value: true });
exports.graphqlWithAuth =
  exports.createOctokit =
  exports.createGraphqlClient =
  exports.saveEtag =
  exports.loadEtag =
  exports.ensureDataDir =
  exports.ETAG_FILE_PATH =
  exports.octokit =
    void 0;
exports.fetchReposBatchWithRetry = fetchReposBatchWithRetry;
var rest_1 = require("@octokit/rest");
var fs = __importStar(require("fs"));
var path = __importStar(require("path"));
var dotenv = __importStar(require("dotenv"));
var graphql_1 = require("@octokit/graphql");
// Load environment variables from .env file
dotenv.config();
// Initialize Octokit for type definitions
exports.octokit = new rest_1.Octokit();
exports.ETAG_FILE_PATH = path.join(process.cwd(), "data", "github_etag.json");
// Ensure the data directory exists
var ensureDataDir = function () {
  var dataDir = path.join(process.cwd(), "data");
  if (!fs.existsSync(dataDir)) {
    fs.mkdirSync(dataDir, { recursive: true });
  }
};
exports.ensureDataDir = ensureDataDir;
// Load etag from file if it exists
var loadEtag = function () {
  try {
    if (fs.existsSync(exports.ETAG_FILE_PATH)) {
      var data = JSON.parse(fs.readFileSync(exports.ETAG_FILE_PATH, "utf8"));
      return data.etag;
    }
  } catch (error) {
    console.warn("Failed to load etag:", error);
  }
  return null;
};
exports.loadEtag = loadEtag;
// Save etag to file
var saveEtag = function (etag) {
  try {
    (0, exports.ensureDataDir)();
    fs.writeFileSync(
      exports.ETAG_FILE_PATH,
      JSON.stringify({
        etag: etag,
        updated: new Date().toISOString(),
      }),
    );
  } catch (error) {
    console.error("Failed to save etag:", error);
  }
};
exports.saveEtag = saveEtag;
var createGraphqlClient = function () {
  return graphql_1.graphql.defaults({
    headers: {
      authorization: "token ".concat(process.env.GITHUB_TOKEN),
    },
  });
};
exports.createGraphqlClient = createGraphqlClient;
// Create a new authenticated Octokit instance
var createOctokit = function () {
  return new rest_1.Octokit({
    auth: process.env.GITHUB_TOKEN,
  });
};
exports.createOctokit = createOctokit;
// Create a typed GraphQL client
exports.graphqlWithAuth = graphql_1.graphql.defaults({
  headers: {
    authorization: "token ".concat(process.env.GITHUB_TOKEN),
  },
});
// Helper function to fetch repository data
function fetchReposBatchWithRetry(repos_1) {
  return __awaiter(this, arguments, void 0, function (repos, retries) {
    var queryFields, query, _loop_1, attempt, state_1;
    if (retries === void 0) {
      retries = 3;
    }
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          queryFields = repos
            .map(function (repo, i) {
              return "\n      repo"
                .concat(i, ': repository(owner: "')
                .concat(repo.owner, '", name: "')
                .concat(
                  repo.name,
                  '") {\n        description\n        primaryLanguage {\n          name\n        }\n        repositoryTopics(first: 10) {\n          nodes {\n            topic {\n              name\n            }\n          }\n        }\n        owner {\n          login\n          id\n          __typename\n          url\n          avatarUrl\n        }\n        stargazerCount\n        forkCount\n        watchers {\n          totalCount\n        }\n        issues(states: OPEN) {\n          totalCount\n        }\n        createdAt\n        homepageUrl\n        organization {\n          id\n          url\n          login\n        }\n      }\n    ',
                );
            })
            .join("\n");
          query = "\n    query {\n      ".concat(queryFields, "\n    }\n  ");
          _loop_1 = function (attempt) {
            var response, repoDataMap_1, error_1, waitTime_1;
            return __generator(this, function (_b) {
              switch (_b.label) {
                case 0:
                  _b.trys.push([0, 2, , 5]);
                  return [4 /*yield*/, (0, exports.graphqlWithAuth)(query)];
                case 1:
                  response = _b.sent();
                  repoDataMap_1 = new Map();
                  Object.entries(response).forEach(function (_a, index) {
                    var key = _a[0],
                      data = _a[1];
                    if (data.repository) {
                      var fullName = ""
                        .concat(repos[index].owner, "/")
                        .concat(repos[index].name);
                      repoDataMap_1.set(fullName, data.repository);
                    }
                  });
                  return [2 /*return*/, { value: repoDataMap_1 }];
                case 2:
                  error_1 = _b.sent();
                  if (!(error_1.status === 403 && attempt < retries))
                    return [3 /*break*/, 4];
                  waitTime_1 = Math.pow(2, attempt) * 1000;
                  return [
                    4 /*yield*/,
                    new Promise(function (resolve) {
                      return setTimeout(resolve, waitTime_1);
                    }),
                  ];
                case 3:
                  _b.sent();
                  return [2 /*return*/, "continue"];
                case 4:
                  throw error_1;
                case 5:
                  return [2 /*return*/];
              }
            });
          };
          attempt = 1;
          _a.label = 1;
        case 1:
          if (!(attempt <= retries)) return [3 /*break*/, 4];
          return [5 /*yield**/, _loop_1(attempt)];
        case 2:
          state_1 = _a.sent();
          if (typeof state_1 === "object") return [2 /*return*/, state_1.value];
          _a.label = 3;
        case 3:
          attempt++;
          return [3 /*break*/, 1];
        case 4:
          throw new Error("Max retries exceeded");
      }
    });
  });
}
//# sourceMappingURL=utils.js.map
