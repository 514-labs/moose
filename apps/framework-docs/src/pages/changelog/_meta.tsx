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
    title: "Changelog",
    theme: {
      breadcrumb: false,
    },
  },
  upcoming: { display: "hidden" }, // This hides it from sidebar/navigation
};

// Process the raw meta object to generate the final meta object with proper rendering
export default render(rawMeta);
