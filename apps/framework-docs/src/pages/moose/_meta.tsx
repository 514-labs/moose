import { render } from "@/components";
import { LanguageSwitcher } from "@/components/language-switcher";
// Raw meta object - more concise without repetitive rendering logic
const rawMeta = {
  // "--Select Language--": {
  //   type: "separator",
  //   title: (
  //     <div className="flex flex-col gap-2">
  //       <p>Language:</p>
  //       <LanguageSwitcher />
  //     </div>
  //   )

  // },
  // First Contact - Essential for newcomers
  "--Getting Started--": {
    type: "separator",
  },
  index: {
    title: "Introduction",
    theme: {
      breadcrumb: false,
    },
  },
  quickstart: "Quick Start",
  architecture: "Architecture Overview",
  "project-structure": "Project Structure",

  // Builder's Guide - Task-oriented approach
  "--Building with Moose--": {
    type: "separator",
  },
  "data-modeling": "Data Modeling",
  ingestion: "Ingesting Data via APIs",
  streams: "Stream Processing",
  "olap-table": "Working with Database Tables",
  "materialized-views": "Transforming Data in your Database",
  "consumption-apis": "Consuming Data via APIs",
  workflows: "Workflow Orchestration",

  // Practical Guidance
  "--Guides--": {
    type: "separator",
  },
  deploying: "Self-Hosted Deployment",
  monitoring: "Monitoring & Observability",
  "data-flow": "Data Flow",

  // Reference & Resources
  "--Reference--": {
    type: "separator",
  },
  "api-reference": "API Reference",
  "moose-cli": "CLI Reference",
  "metrics-console": "Metrics Console",

  // Help & Support
  "--Help--": {
    type: "separator",
  },
  faqs: "FAQs",

  // Hidden pages
  v1: {
    display: "hidden",
  },
};

// Process the raw meta object to generate the final meta object with proper rendering
export default render(rawMeta);
