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
  GanttChart,
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
  __modules__: {
    type: "separator",
    title: "Modules",
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
  apis: {
    title: "APIs",
    Icon: Code,
    isMoose: true,
  },
  __deployment__: {
    type: "separator",
    title: "Deployment Tools & Guides",
  },
  migrate: {
    title: "Migrate",
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
