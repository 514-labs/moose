import { render } from "@/components";

const rawMeta = {
  "ingest-api": {
    title: "Ingest New Data",
  },
  "analytics-api": {
    title: "Expose Analytics",
  },
  "trigger-api": {
    title: "Trigger Workflows",
  },
  auth: {
    title: "Securing API Endpoints",
  },
  __client__: {
    type: "separator",
    title: "Client Libraries",
  },
  "openapi-sdk": {
    title: "OpenAPI SDK",
  },
  "admin-api": {
    title: "Admin APIs",
    display: "hidden",
  },
};

export default render(rawMeta);
