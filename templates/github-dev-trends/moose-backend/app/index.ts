import { IGhEvent, IRepoStarEvent } from "./ingest/models";
import { transformGhEvent } from "./ingest/transform";
export * from "./apis/topicTimeseries";

import { IngestPipeline } from "@514labs/moose-lib";

export const GhEvent = new IngestPipeline<IGhEvent>("GhEvent", {
  ingest: true,
  table: true,
  stream: true,
});

export const RepoStarEvent = new IngestPipeline<IRepoStarEvent>("RepoStar", {
  ingest: false,
  stream: true,
  table: true,
});

GhEvent.stream!.addTransform(RepoStarEvent.stream!, transformGhEvent);
