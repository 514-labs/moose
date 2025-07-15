import { render } from "@/components";

const meta = {
  "data-modeling": "Data Modeling",
  ingestion: "Ingesting Data via API",
  streams: "Processing Data in Streams",
  "olap-table": "Storing Data in ClickHouse",
  "materialized-views": "Transforming Data in ClickHouse",
  "consumption-apis": "Querying Data via API",
  workflows: "Orchestrating Data Workflows",
  "dead-letter-queues": "Handling Data Errors",
};

export default render(meta);
