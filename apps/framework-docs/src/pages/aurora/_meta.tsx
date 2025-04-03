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
  "prompting-best-practices": "Prompting best practices",
} as const;

export default render(meta);
