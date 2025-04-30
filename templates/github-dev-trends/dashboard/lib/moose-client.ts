import {
  DefaultApi,
  Configuration,
  ConsumptionTopicTimeseriesGetRequest,
  ConsumptionTopicTimeseriesGet200ResponseInner,
} from "api-client";

export type TopicTimeseriesRequest = ConsumptionTopicTimeseriesGetRequest;
export type TopicTimeseriesResponse =
  ConsumptionTopicTimeseriesGet200ResponseInner;

const mooseUrl = process.env.MOOSE_URL || "http://localhost:4000";

const apiConfig = new Configuration({
  basePath: mooseUrl,
});
const mooseClient = new DefaultApi(apiConfig);

export default mooseClient;
