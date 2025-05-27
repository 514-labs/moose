import { render } from "@/components";

const meta = {
  index: {
    title: "Introduction",
    theme: {
      breadcrumb: false,
    },
  },
  quickstart: "Quickstart",
  Install: "Installation",
  host_setup: "Host setup guide",
  "cli-reference": "CLI reference",
  "tool-reference": "Tool reference",
  "data-collection-policy": "Data collection policy",
} as const;

export default render(meta);
