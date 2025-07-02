import { render } from "@/components";

const meta = {
  ingest: {
    title: "Ingest",
    theme: {
      breadcrumb: false,
    },
  },
  "model-data": {
    title: "Model Data Engineering",
    theme: {
      breadcrumb: false,
    },
  },
  mvs: {
    title: "MVs",
    theme: {
      breadcrumb: false,
    },
  },
  dlqs: {
    title: "DLQs",
    theme: {
      breadcrumb: false,
    },
  },
  egress: {
    title: "Egress",
    theme: {
      breadcrumb: false,
    },
  },
};

export default render(meta);
