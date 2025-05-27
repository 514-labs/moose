import { render } from "@/components";

const meta = {
  "data-modeling": "Data Modeling",
  ingestion: "Ingesting Data into Analytics Storage",
  streams: "Stream Processing",
  "olap-table": "Working with OLAP Tables",
  "materialized-views": "Transforming Data in-Database",
  "consumption-apis": "Exposing Analytics via API",
  workflows: "Scheduling & Triggering Workflows",
  "dead-letter-queues": "Error Handling With Dead Letter Queues",
};

export default render(meta);
