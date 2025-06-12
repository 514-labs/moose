"use strict";
var __assign = (this && this.__assign) || function () {
    __assign = Object.assign || function(t) {
        for (var s, i = 1, n = arguments.length; i < n; i++) {
            s = arguments[i];
            for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
                t[p] = s[p];
        }
        return t;
    };
    return __assign.apply(this, arguments);
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.transformGhEvent = transformGhEvent;
var models_1 = require("./models");
var utils_1 = require("../utils");
var moose_lib_1 = require("@514labs/moose-lib");
var octokit = (0, utils_1.createOctokit)();
function transformGhEvent(event) {
    return __awaiter(this, void 0, void 0, function () {
        var repo, repoData;
        var _a, _b, _c, _d, _e, _f, _g, _h, _j, _k;
        return __generator(this, function (_l) {
            switch (_l.label) {
                case 0:
                    if (!(event.eventType == models_1.GitHubEventType.Watch)) return [3 /*break*/, 2];
                    (0, moose_lib_1.cliLog)({
                        action: "fetching repo",
                        message: event.repoName,
                    });
                    return [4 /*yield*/, octokit.rest.repos.get({
                            owner: event.repoOwner,
                            repo: event.repoName,
                        })];
                case 1:
                    repo = _l.sent();
                    (0, moose_lib_1.cliLog)({
                        action: "repo fetched",
                        message: repo.data.name,
                    });
                    repoData = repo.data;
                    return [2 /*return*/, __assign(__assign({}, event), { repoDescription: (_a = repoData.description) !== null && _a !== void 0 ? _a : "", repoTopics: (_b = repoData.topics) !== null && _b !== void 0 ? _b : [], repoLanguage: (_c = repoData.language) !== null && _c !== void 0 ? _c : "", repoStars: repoData.stargazers_count, repoForks: repoData.forks_count, repoWatchers: repoData.watchers_count, repoOpenIssues: repoData.open_issues_count, repoCreatedAt: repoData.created_at
                                ? new Date(repoData.created_at)
                                : new Date(), repoOwnerLogin: repoData.owner.login, repoOwnerId: repoData.owner.id, repoOwnerUrl: repoData.owner.url, repoOwnerAvatarUrl: repoData.owner.avatar_url, repoOwnerType: repoData.owner.type, repoOrgId: (_e = (_d = repoData.organization) === null || _d === void 0 ? void 0 : _d.id) !== null && _e !== void 0 ? _e : 0, repoOrgUrl: (_g = (_f = repoData.organization) === null || _f === void 0 ? void 0 : _f.url) !== null && _g !== void 0 ? _g : "", repoOrgLogin: (_j = (_h = repoData.organization) === null || _h === void 0 ? void 0 : _h.login) !== null && _j !== void 0 ? _j : "", repoHomepage: (_k = repoData.homepage) !== null && _k !== void 0 ? _k : "" })];
                case 2: return [2 /*return*/];
            }
        });
    });
}
//# sourceMappingURL=transform.js.map