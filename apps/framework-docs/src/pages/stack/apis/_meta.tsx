import { render } from "@/components";

const rawMeta = {
  "ingest-data": {
    title: "Configure Ingest Endpoints",
  },
  "egress-data": {
    title: "Expose Queries as APIs",
  },
  auth: {
    title: "Securing API Endpoints",
  },
};

export const meta = render(rawMeta);
