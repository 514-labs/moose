import { IGhEvent, IRepoStarEvent } from "./ingest/models";
import { transformGhEvent } from "./ingest/transform";
import {
  getTopicTimeseries,
  QueryParams,
  ResponseBody,
} from "./apis/topicTimeseries";

import { IngestPipeline, ConsumptionApi } from "@514labs/moose-lib";

// Pipeline to receive raw events from the Github API
export const GhEvent = new IngestPipeline<IGhEvent>("GhEvent", {
  ingest: true,
  table: true,
  stream: true,
});

// Pipeline to receive transformed events from the GhEvent pipeline
export const RepoStarEvent = new IngestPipeline<IRepoStarEvent>("RepoStar", {
  ingest: false,
  stream: true,
  table: true,
});

// Streaming transformation to transform the raw GhEvent stream into an enriched RepoStarEvent stream
GhEvent.stream!.addTransform(RepoStarEvent.stream!, transformGhEvent);

// Consumption API to get a timeseries of the top `n` topics over a given interval
export const topicTimeseriesApi = new ConsumptionApi<
  QueryParams,
  ResponseBody[]
>("topicTimeseries", getTopicTimeseries);
