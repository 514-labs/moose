import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";
import {
  FetchEventsOutput,
  createOctokit,
  RepoType,
  RepoRequestOptions,
  RepoResponseType,
} from "../utils";
import { WatchEvent, WatchEventWithRepo } from "../../datamodels/WatchEvent";
import { cliLog } from "@514labs/moose-lib";
// The initial input data and data passed between tasks can be
// defined in the task function parameter
const load: TaskFunction = async (input?: FetchEventsOutput) => {
  // The body of your script goes here
  console.log("Hello world from load");

  if (!input || input.noNewEvents) {
    console.log("No new events to load");
    return {
      task: "load",
      data: {
        eventsLoaded: 0,
      },
    };
  }

  console.log(`Loading ${input.count} events from ${input.fetchedAt}`);

  const octokit = createOctokit();
  // Process the events here
  // input.events contains the GitHub events data
  for (const event of input.events) {
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
      } as RepoRequestOptions);

      const repoData = repo.data as RepoType;

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

      cliLog({ action: "load", message: JSON.stringify(mooseEventWithRepo) });
    }
  }

  // The return value is the output of the script.
  // The return value should be a dictionary with at least:
  // - task: the task name (e.g., "extract", "transform")
  // - data: the actual data being passed to the next task
  return {
    task: "load",
    data: {
      eventsLoaded: input.count,
      loadedAt: new Date().toISOString(),
    },
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
