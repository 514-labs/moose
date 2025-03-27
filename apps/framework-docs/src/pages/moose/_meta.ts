export default {
  // First Contact - Essential for newcomers
  "--Getting Started--": {
    type: "separator",
    title: "Getting Started",
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
    title: "Building with Moose",
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
    title: "Guides",
  },
  deploying: "Self-Hosted Deployment",
  monitoring: "Monitoring & Observability",
  "data-flow": "Data Flow",

  // Reference & Resources
  "--Reference--": {
    type: "separator",
    title: "Reference",
  },
  "api-reference": "API Reference",
  "moose-cli": "CLI Reference",
  "metrics-console": "Metrics Console",

  // Help & Support
  "--Help--": {
    type: "separator",
    title: "Help & Support",
  },
  faqs: "FAQs",

  // Hidden pages
  v1: {
    display: "hidden",
  },
} as const;
