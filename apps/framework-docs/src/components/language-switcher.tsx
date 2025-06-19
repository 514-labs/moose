import React from "react";
import { useLanguage } from "./LanguageContext";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Heading, HeadingLevel } from "@/components/typography";

export function LanguageSwitcher() {
  const { language, setLanguage } = useLanguage();

  const otherLanguage = language === "python" ? "typescript" : "python";

  return (
    <div className="sticky top-[64px] z-50 py-2 bg-background flex">
      <Select
        value={language}
        onValueChange={(value: "typescript" | "python") => setLanguage(value)}
      >
        <SelectTrigger className="w-[auto]">
          <SelectValue placeholder="Select Language" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="typescript">TypeScript</SelectItem>
          <SelectItem value="python">Python</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}
