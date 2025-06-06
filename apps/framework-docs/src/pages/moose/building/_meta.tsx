import { render } from "@/components";

const meta = {
  "data-modeling": "Data Modeling",
  ingestion: "Ingesting Data into Analytics Storage",
  streams: "Stream Processing",
  "olap-table": "Working with OLAP Tables",
  "materialized-views": "Transforming Data in-Database",
  "consumption-apis": "Exposing Analytics via API",
  workflows: "Scheduling & Triggering Workflows",
  "workflows-2": {
    title: "Scheduling & Triggering Workflows 2.0",
    display: "hidden",
  },
  "dead-letter-queues": "Error Handling With Dead Letter Queues",
};

export default render(meta);
