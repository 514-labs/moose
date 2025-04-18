import {
  GhEvent,
  RepoStarEvent,
  GitHubEventType,
  IRepoStarEvent,
  IGhEvent,
} from "./models";
import { createOctokit, RepoResponseType } from "../utils";
import { cliLog } from "@514labs/moose-lib";

const octokit = createOctokit();

const ghStream = GhEvent.stream!;
const repoStarStream = RepoStarEvent.stream!;

async function transformGhEvent(
  event: IGhEvent,
): Promise<IRepoStarEvent | undefined> {
  // Only transform watch events for now
  if (event.eventType == GitHubEventType.Watch) {
    cliLog({
      action: "fetching repo",
      message: event.repoName,
    });
    const repo: RepoResponseType = await octokit.rest.repos.get({
      owner: event.repoOwner,
      repo: event.repoName,
    });

    cliLog({
      action: "repo fetched",
      message: repo.data.name,
    });

    const repoData = repo.data;
    return {
      ...event,
      repoDescription: repoData.description ?? undefined,
      repoTopics: repoData.topics ?? undefined,
      repoLanguage: repoData.language ?? undefined,
      repoStars: repoData.stargazers_count,
      repoForks: repoData.forks_count,
      repoWatchers: repoData.watchers_count,
      repoOpenIssues: repoData.open_issues_count,
      repoCreatedAt: repoData.created_at
        ? new Date(repoData.created_at)
        : undefined,
      repoOwnerLogin: repoData.owner.login,
      repoOwnerId: repoData.owner.id,
      repoOwnerUrl: repoData.owner.url,
      repoOwnerAvatarUrl: repoData.owner.avatar_url,
      repoOwnerType: repoData.owner.type,
      repoOrgId: repoData.organization?.id,
      repoOrgUrl: repoData.organization?.url,
      repoOrgLogin: repoData.organization?.login,
      // repoHomepage: repoData.homepage,
    };
  }
}

ghStream.addTransform(repoStarStream, transformGhEvent);
