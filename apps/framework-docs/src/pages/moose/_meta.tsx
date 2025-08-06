import { render } from "@/components";
import { LanguageSwitcher } from "@/components/language-switcher";
import { SmallText } from "@/components/typography";
import {
  ArrowRight,
  HandMetal,
  Code,
  BookOpen,
  Rocket,
  BookMarked,
  History,
  Database,
  Workflow,
  Waves,
  GitCompare,
  HelpCircle,
  Settings,
  ChartBar,
  Hammer,
  Terminal,
  Database,
  Workflow,
  Waves,
  GitCompare,
  HelpCircle,
  Settings,
  ChartBar,
  Hammer,
  Terminal,
} from "lucide-react";

// Raw meta object - more concise without repetitive rendering logic
const rawMeta = {
  index: {
    title: "Welcome to Moose",
    theme: {
      breadcrumb: false,
    },
  },
  "getting-started": {
    title: "Getting Started",
    Icon: HandMetal,
  },
  migrate: {
    title: "Migrate",
    Icon: ArrowRight,
    isMoose: true,
  },
  olap: {
    title: "OLAP",
    Icon: Database,
    isMoose: true,
  },
  streaming: {
    title: "Streaming",
    Icon: Waves,
    isMoose: true,
  },
  workflows: {
    title: "Workflows",
    Icon: Workflow,
    isMoose: true,
  },
  // Builder's Guide - Task-oriented approach
  building: {
    title: "Developing",
    Icon: Code,
  },
  __deployment__: {
    type: "separator",
    title: "Deployment Tools & Guides",
  },
  planning: {
    title: "Plan",
    Icon: GitCompare,
    isMoose: true,
  },
  metrics: {
    title: "Metrics",
    Icon: ChartBar,
    isMoose: true,
  },
  deploying: {
    title: "Self-Hosting",
    Icon: Hammer,
  },
  __reference__: {
    type: "separator",
    title: "Reference",
  },
  reference: {
    title: "API Reference",
    Icon: BookMarked,
  },
  "moose-cli": {
    title: "CLI",
    Icon: Terminal,
  },
  configuration: {
    title: "Project Configuration",
    Icon: Settings,
  },
  help: {
    title: "Help",
    Icon: HelpCircle,
  },
  changelog: {
    title: "Changelog",
    Icon: History,
  },
};

// Process the raw meta object to generate the final meta object with proper rendering
export default render(rawMeta);
