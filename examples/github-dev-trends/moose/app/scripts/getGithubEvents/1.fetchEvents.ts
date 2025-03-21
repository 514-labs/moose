import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";
import { Octokit } from "@octokit/rest";
import { WatchEventWithRepo } from "datamodels/WatchEvents";
import { WatchEvent } from "datamodels/WatchEvents";

const octokit = new Octokit({
  auth: process.env.GITHUB_TOKEN,
});

// The initial input data and data passed between tasks can be
// defined in the task function parameter
const load: TaskFunction = async (input?: any) => {
  const responses = await octokit.paginate.iterator(
    octokit.activity.listPublicEvents,
    {
      per_page: 100,
    },
  );

  for await (const response of responses) {
    for (const event of response.data) {
      if (event.type === "WatchEvent") {
        const mooseEvent = {
          eventId: event.id,
          actorLogin: event.actor.login,
          actorId: event.actor.id,
          actorUrl: event.actor.url,
          actorAvatarUrl: event.actor.avatar_url,
          repoName: event.repo.name,
          repoUrl: event.repo.url,
          repoId: event.repo.id,
          createdAt: event.created_at ? new Date(event.created_at) : null,
        } as WatchEvent;

        const repo = await octokit.rest.repos.get({
          owner: event.repo.name.split("/")[0],
          repo: event.repo.name.split("/")[1],
        });

        const repoData = repo.data;

        const mooseEventWithRepo = {
          ...mooseEvent,
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
          repoOrgId: repoData.organization?.id,
          repoOrgUrl: repoData.organization?.url,
          repoOrgLogin: repoData.organization?.login,
          repoHomepage: repoData.homepage,
        } as WatchEventWithRepo;

        await fetch("http://localhost:4000/ingest/WatchEventWithRepo", {
          method: "POST",
          body: JSON.stringify(mooseEventWithRepo),
        });
      }
    }
  }

  return {
    task: "load",
    data: {},
  };
};

export default function createTask() {
  return {
    task: load,
    config: {
      retries: 3,
    },
  } as TaskDefinition;
}
