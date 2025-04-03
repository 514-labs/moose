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
};

// Process the raw meta object to generate the final meta object with proper rendering
export default render(rawMeta);
