import { render } from "@/components";
import { HandMetal, BookMarked, History } from "lucide-react";

const meta = {
  index: {
    title: "Introduction",
    theme: {
      breadcrumb: false,
    },
  },
  quickstart: {
    title: "Quickstart Guides",
    Icon: HandMetal,
  },
  reference: {
    title: "Reference",
    Icon: BookMarked,
  },
  "data-collection-policy": {
    title: "Data collection policy",
    Icon: History,
  },
} as const;

export default render(meta);
