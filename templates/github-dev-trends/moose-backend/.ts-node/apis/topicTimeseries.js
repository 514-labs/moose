"use strict";
var __makeTemplateObject = (this && this.__makeTemplateObject) || function (cooked, raw) {
    if (Object.defineProperty) { Object.defineProperty(cooked, "raw", { value: raw }); } else { cooked.raw = raw; }
    return cooked;
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
exports.getTopicTimeseries = getTopicTimeseries;
var index_1 = require("../index");
function getTopicTimeseries(_a, _b) {
    return __awaiter(this, arguments, void 0, function (_c, _d) {
        var RepoTable, cols, intervalMap, query, resultSet;
        var _e = _c.interval, interval = _e === void 0 ? "minute" : _e, _f = _c.limit, limit = _f === void 0 ? 10 : _f, _g = _c.exclude, exclude = _g === void 0 ? "" : _g;
        var client = _d.client, sql = _d.sql;
        return __generator(this, function (_h) {
            switch (_h.label) {
                case 0:
                    RepoTable = index_1.RepoStarEvent.table;
                    cols = RepoTable.columns;
                    intervalMap = {
                        hour: {
                            select: sql(templateObject_1 || (templateObject_1 = __makeTemplateObject(["toStartOfHour(", ") AS time"], ["toStartOfHour(", ") AS time"])), cols.createdAt),
                            groupBy: sql(templateObject_2 || (templateObject_2 = __makeTemplateObject(["GROUP BY time, topic"], ["GROUP BY time, topic"]))),
                            orderBy: sql(templateObject_3 || (templateObject_3 = __makeTemplateObject(["ORDER BY time, totalEvents DESC"], ["ORDER BY time, totalEvents DESC"]))),
                            limit: sql(templateObject_4 || (templateObject_4 = __makeTemplateObject(["LIMIT ", " BY time"], ["LIMIT ", " BY time"])), limit),
                        },
                        day: {
                            select: sql(templateObject_5 || (templateObject_5 = __makeTemplateObject(["toStartOfDay(", ") AS time"], ["toStartOfDay(", ") AS time"])), cols.createdAt),
                            groupBy: sql(templateObject_6 || (templateObject_6 = __makeTemplateObject(["GROUP BY time, topic"], ["GROUP BY time, topic"]))),
                            orderBy: sql(templateObject_7 || (templateObject_7 = __makeTemplateObject(["ORDER BY time, totalEvents DESC"], ["ORDER BY time, totalEvents DESC"]))),
                            limit: sql(templateObject_8 || (templateObject_8 = __makeTemplateObject(["LIMIT ", " BY time"], ["LIMIT ", " BY time"])), limit),
                        },
                        minute: {
                            select: sql(templateObject_9 || (templateObject_9 = __makeTemplateObject(["toStartOfFifteenMinutes(", ") AS time"], ["toStartOfFifteenMinutes(", ") AS time"])), cols.createdAt),
                            groupBy: sql(templateObject_10 || (templateObject_10 = __makeTemplateObject(["GROUP BY time, topic"], ["GROUP BY time, topic"]))),
                            orderBy: sql(templateObject_11 || (templateObject_11 = __makeTemplateObject(["ORDER BY time, totalEvents DESC"], ["ORDER BY time, totalEvents DESC"]))),
                            limit: sql(templateObject_12 || (templateObject_12 = __makeTemplateObject(["LIMIT ", " BY time"], ["LIMIT ", " BY time"])), limit),
                        },
                    };
                    query = sql(templateObject_15 || (templateObject_15 = __makeTemplateObject(["\n            SELECT\n                time,\n                arrayMap(\n                    (topic, events, repos, users) -> map(\n                        'topic', topic,\n                        'eventCount', toString(events),\n                        'uniqueRepos', toString(repos),\n                        'uniqueUsers', toString(users)\n                    ),\n                    groupArray(topic),\n                    groupArray(totalEvents),\n                    groupArray(uniqueReposCount),\n                    groupArray(uniqueUsersCount)\n                ) AS topicStats\n            FROM (\n                SELECT\n                    ", ",\n                    arrayJoin(", ") AS topic,\n                    count() AS totalEvents,\n                    uniqExact(", ") AS uniqueReposCount,\n                    uniqExact(", ") AS uniqueUsersCount\n                FROM ", "\n                WHERE length(", ") > 0\n                ", "\n                ", "\n                ", "\n                ", "\n            )\n            GROUP BY time\n            ORDER BY time;\n        "], ["\n            SELECT\n                time,\n                arrayMap(\n                    (topic, events, repos, users) -> map(\n                        'topic', topic,\n                        'eventCount', toString(events),\n                        'uniqueRepos', toString(repos),\n                        'uniqueUsers', toString(users)\n                    ),\n                    groupArray(topic),\n                    groupArray(totalEvents),\n                    groupArray(uniqueReposCount),\n                    groupArray(uniqueUsersCount)\n                ) AS topicStats\n            FROM (\n                SELECT\n                    ", ",\n                    arrayJoin(", ") AS topic,\n                    count() AS totalEvents,\n                    uniqExact(", ") AS uniqueReposCount,\n                    uniqExact(", ") AS uniqueUsersCount\n                FROM ", "\n                WHERE length(", ") > 0\n                ", "\n                ", "\n                ", "\n                ", "\n            )\n            GROUP BY time\n            ORDER BY time;\n        "])), intervalMap[interval].select, cols.repoTopics, cols.repoId, cols.actorId, index_1.RepoStarEvent.table, cols.repoTopics, exclude ? sql(templateObject_13 || (templateObject_13 = __makeTemplateObject(["AND arrayAll(x -> x NOT IN (", "), ", ")"], ["AND arrayAll(x -> x NOT IN (", "), ", ")"])), exclude, cols.repoTopics) : sql(templateObject_14 || (templateObject_14 = __makeTemplateObject([""], [""]))), intervalMap[interval].groupBy, intervalMap[interval].orderBy, intervalMap[interval].limit);
                    return [4 /*yield*/, client.query.execute(query)];
                case 1:
                    resultSet = _h.sent();
                    return [4 /*yield*/, resultSet.json()];
                case 2: return [2 /*return*/, _h.sent()];
            }
        });
    });
}
var templateObject_1, templateObject_2, templateObject_3, templateObject_4, templateObject_5, templateObject_6, templateObject_7, templateObject_8, templateObject_9, templateObject_10, templateObject_11, templateObject_12, templateObject_13, templateObject_14, templateObject_15;
//# sourceMappingURL=topicTimeseries.js.map