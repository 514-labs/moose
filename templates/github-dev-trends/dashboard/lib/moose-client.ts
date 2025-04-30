import { DefaultApi, Configuration } from "api-client";

const mooseUrl = process.env.NEXT_PUBLIC_MOOSE_URL || "http://localhost:4000";

const apiConfig = new Configuration({
  basePath: mooseUrl,
});
const mooseClient = new DefaultApi(apiConfig);

export default mooseClient;
