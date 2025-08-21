import { IGhEvent, IRepoStarEvent } from "./ingest/models";
import {
  getTopicTimeseries,
  QueryParams,
  ResponseBody,
} from "./apis/topicTimeseries";

import {
  IngestPipeline,
  ConsumptionApi,
  Key,
  ConsumptionUtil,
} from "@514labs/moose-lib/browserCompatible";

// Pipeline to receive raw events from the Github API
export const GhEvent = new IngestPipeline<IGhEvent>("GhEvent", {
  ingestApi: true,
  table: true,
  stream: true,
});

// Pipeline to receive transformed events from the GhEvent pipeline
export const RepoStarEvent = new IngestPipeline<IRepoStarEvent>("RepoStar", {
  ingestApi: false,
  stream: true,
  table: true,
});

// Consumption API to get a timeseries of the top `n` topics over a given interval
export const topicTimeseriesApi = new ConsumptionApi<
  QueryParams,
  ResponseBody[]
>("topicTimeseries", getTopicTimeseries);
