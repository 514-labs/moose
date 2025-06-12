"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GitHubEventType = void 0;
var GitHubEventType;
(function (GitHubEventType) {
    GitHubEventType["Watch"] = "WatchEvent";
    GitHubEventType["Push"] = "PushEvent";
    GitHubEventType["Issue"] = "IssuesEvent";
    GitHubEventType["IssueComment"] = "IssueCommentEvent";
    GitHubEventType["PullRequest"] = "PullRequestEvent";
    GitHubEventType["PullRequestReview"] = "PullRequestReviewEvent";
    GitHubEventType["Create"] = "CreateEvent";
    GitHubEventType["Delete"] = "DeleteEvent";
    GitHubEventType["PRComment"] = "PullRequestReviewCommentEvent";
    GitHubEventType["Fork"] = "ForkEvent";
    GitHubEventType["Member"] = "MemberEvent";
    GitHubEventType["Release"] = "ReleaseEvent";
})(GitHubEventType || (exports.GitHubEventType = GitHubEventType = {}));
//# sourceMappingURL=models.js.map