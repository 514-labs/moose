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
  deploying: {
    title: "Deploying",
    Icon: Rocket,
  },
  reference: {
    title: "Reference",
    Icon: BookMarked,
  },
  changelog: {
    title: "Changelog",
    Icon: History,
  },
  streaming: {
    display: "hidden",
  },
};

// Process the raw meta object to generate the final meta object with proper rendering
export default render(rawMeta);
