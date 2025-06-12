import { render } from "@/components";

const meta = {
  index: "Overview",
  "data-modeling": "Data Modeling",
  ingestion: "Ingestion APIs",
  streams: "Stream Processing",
  "olap-table": "OLAP Tables",
  "materialized-views": "Materialized Views",
  workflows: "Workflows & Tasks",
  "consumption-apis": "Consumption APIs",
  "dead-letter-queues": "Error Handling",
  "workflows-2": {
    title: "Workflows V2 (Legacy)",
    display: "hidden",
  },
};

export default render(meta);
