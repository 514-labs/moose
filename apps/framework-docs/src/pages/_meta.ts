export default {
  "--Get Started--": {
    type: "separator",
    title: "Get Started",
  },
  index: {
    title: "Introduction",
    theme: {
      breadcrumb: false,
    },
  },
  quickstart: "Quick Start",
  "project-structure": "Project Structure",
  architecture: "Architecture",
  "learn-moose": "Tutorial",
  "--Develop--": {
    type: "separator",
    title: "Develop",
  },
  "data-models": "Data Modeling",
  "ingest-data": "Ingesting Data",
  "stream-processing": "Stream Processing",
  "db-processing": "Database Processing",
  "consumption-apis": "Analytics APIs",
  planning: "Infrastructure Planning",
  workflows: "Orchestrating Workflows",
  "metrics-console": "Monitoring Performance",
  "--Deploy--": {
    type: "separator",
    title: "Deploy",
  },
  deploying: "Deploying your App",
  "--Reference--": {
    type: "separator",
    title: "Reference",
  },
  "moose-cli": "Moose CLI",
  templates: {
    display: "hidden",
  },
  faqs: "FAQs",
  "usage-data": "Privacy",
  "schema-evolution": {
    display: "hidden",
  },
  "data-modeling-v2": {
    display: "hidden",
  },
} as const;
