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
Object.defineProperty(exports, "__esModule", { value: true });
exports.createOctokit =
  exports.saveEtag =
  exports.loadEtag =
  exports.ensureDataDir =
  exports.ETAG_FILE_PATH =
  exports.octokit =
    void 0;
var rest_1 = require("@octokit/rest");
var fs = __importStar(require("fs"));
var path = __importStar(require("path"));
var dotenv = __importStar(require("dotenv"));
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
// Create a new authenticated Octokit instance
var createOctokit = function () {
  return new rest_1.Octokit({
    auth: process.env.GITHUB_TOKEN,
  });
};
exports.createOctokit = createOctokit;
//# sourceMappingURL=utils.js.map
