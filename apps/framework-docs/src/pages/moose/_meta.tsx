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
  // "--Select Language--": {
  //   type: "separator",
  //   title: (
  //     <div className="flex flex-col gap-2">
  //       <p>Language:</p>
  //       <LanguageSwitcher />
  //     </div>
  //   )

  // },
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
  "data-flow": {
    display: "hidden",
  },
  "metrics-console": {
    display: "hidden",
  },

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
};

// Process the raw meta object to generate the final meta object with proper rendering
export default render(rawMeta);
