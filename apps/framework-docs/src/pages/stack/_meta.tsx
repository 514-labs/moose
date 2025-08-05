import { render } from "@/components";
import { LanguageSwitcher } from "@/components/language-switcher";
import { SmallText } from "@/components/typography";
import {
  Settings,
  HandMetal,
  Code,
  BookOpen,
  Rocket,
  Terminal,
  Hammer,
  Rss,
  Workflow,
  Database,
  GanttChart,
  ChartBar,
} from "lucide-react";

// Raw meta object - more concise without repetitive rendering logic
const rawMeta = {
  index: {
    title: "Overview",
    theme: {
      breadcrumb: false,
    },
    Icon: GanttChart,
  },
  "getting-started": {
    title: "Getting Started",
    Icon: HandMetal,
  },
  "---fundamentals---": {
    title: "Fundamentals",
    type: "separator",
  },
  runtime: {
    title: "Runtime",
    Icon: Rocket,
    isMoose: true,
  },
  "data-modeling": {
    title: "Data Models",
    Icon: Database,
    isMoose: true,
  },
  // Builder's Guide - Task-oriented approach
  "---modules---": {
    title: "Modules",
    type: "separator",
  },
  olap: {
    title: "OLAP",
    Icon: Database,
    isMoose: true,
  },
  streaming: {
    title: "Streaming",
    Icon: Rss,
    isMoose: true,
  },
  workflows: {
    title: "Workflows",
    Icon: Workflow,
    isMoose: true,
  },
  apis: {
    title: "APIs",
    Icon: Code,
    isMoose: true,
  },
  tools: {
    title: "Tools",
    type: "separator",
  },
  "moose-build": {
    title: "Build",
    Icon: Hammer,
    isMoose: true,
  },
  plan: {
    title: "Plan",
    Icon: BookOpen,
    isMoose: true,
  },
  metrics: {
    title: "Metrics",
    Icon: ChartBar,
    isMoose: true,
  },
  "---client-integrations---": {
    type: "separator",
    title: "Client Integrations",
  },
  "open-api": {
    title: "OpenAPI",
    Icon: Code,
  },

  "---reference---": {
    title: "Reference",
    type: "separator",
  },
  "api-reference": {
    title: "API Reference",
    Icon: BookOpen,
  },
  "moose-cli": {
    title: "CLI",
    Icon: Terminal,
  },
  configuration: {
    title: "Project Configuration",
    Icon: Settings,
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
