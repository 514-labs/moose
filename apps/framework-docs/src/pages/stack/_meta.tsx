import { render } from "@/components";
import { Icons } from "@/components";

// Raw meta object - more concise without repetitive rendering logic
const rawMeta = {
  index: {
    title: "Overview",
    theme: {
      breadcrumb: false,
    },
    Icon: Icons.overview,
  },
  "getting-started": {
    title: "Getting Started",
    Icon: Icons.gettingStarted,
  },
  "---fundamentals---": {
    title: "Fundamentals",
    type: "separator",
  },
  runtime: {
    title: "Runtime",
    Icon: Icons.runtime,
    isMoose: true,
  },
  "data-modeling": {
    title: "Data Models",
    Icon: Icons.dataModels,
    isMoose: true,
  },
  // Builder's Guide - Task-oriented approach
  "---modules---": {
    title: "Modules",
    type: "separator",
  },
  olap: {
    title: "OLAP",
    Icon: Icons.olap,
    isMoose: true,
  },
  streaming: {
    title: "Streaming",
    Icon: Icons.streaming,
    isMoose: true,
  },
  workflows: {
    title: "Workflows",
    Icon: Icons.workflows,
    isMoose: true,
  },
  apis: {
    title: "APIs",
    Icon: Icons.apis,
    isMoose: true,
  },
  tools: {
    title: "Tools",
    type: "separator",
  },
  "moose-build": {
    title: "Build",
    Icon: Icons.build,
    isMoose: true,
  },
  plan: {
    title: "Plan",
    Icon: Icons.plan,
    isMoose: true,
  },
  metrics: {
    title: "Metrics",
    Icon: Icons.metrics,
    isMoose: true,
  },
  "---client-integrations---": {
    type: "separator",
    title: "Client Integrations",
  },
  "open-api": {
    title: "OpenAPI",
  },

  "---reference---": {
    title: "Reference",
    type: "separator",
  },
  "api-reference": {
    title: "API Reference",
    Icon: Icons.apiReference,
  },
  "moose-cli": {
    title: "CLI",
    Icon: Icons.mooseCli,
  },
  configuration: {
    title: "Project Configuration",
    Icon: Icons.configuration,
  },
  help: {
    title: "Help",
    Icon: Icons.help,
  },
  // "client-integrations": {
  //   title: "Client Integrations",
  //   Icon: Code,
  // },
  // reference: {
  //   title: "Reference",
  //   Icon: BookMarked,
  // },
  // changelog: {
  //   title: "Changelog",
  //   Icon: History,
  // },
};

// Process the raw meta object to generate the final meta object with proper rendering
const showSeparatorLine = true;
export default render(rawMeta, showSeparatorLine);
