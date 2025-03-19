import {
  DefaultApi,
  Configuration,
  ConfigurationParameters,
  ConsumptionTopicTimeseriesGetRequest,
  ConsumptionTopicTimeseriesGet200ResponseInner,
} from "api-client";

export type TopicTimeseriesRequest = ConsumptionTopicTimeseriesGetRequest;
export type TopicTimeseriesResponse =
  ConsumptionTopicTimeseriesGet200ResponseInner;

const apiConfig = new Configuration({
  basePath: "http://localhost:4000",
});
const mooseClient = new DefaultApi(apiConfig);

export default mooseClient;
