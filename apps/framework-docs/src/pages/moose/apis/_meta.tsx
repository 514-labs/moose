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
  "admin-api": {
    title: "Admin APIs",
    display: "hidden",
  },
};

export default render(rawMeta);
