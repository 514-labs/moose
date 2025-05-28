import { render } from "@/components";

const meta = {
  index: {
    title: "Introduction",
    theme: {
      breadcrumb: false,
    },
  },
  quickstart: "Quickstart",
  "cli-reference": "CLI reference",
  "tool-reference": "Tool reference",
  "data-collection-policy": "Data collection policy",
} as const;

export default render(meta);
