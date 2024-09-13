import React from "react";
import { useLanguage } from "./LanguageContext";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@514labs/design-system-components/components";
import {
  Heading,
  HeadingLevel,
} from "@514labs/design-system-components/typography";

export function LanguageSwitcher() {
  const { language, setLanguage } = useLanguage();

  const otherLanguage = language === "python" ? "typescript" : "python";

  return (
    <div className="my-5 flex flex-row w-full border-t border-b justify-between">
      <Heading level={HeadingLevel.l5} className="text-captialize">
        Viewing {language}
      </Heading>
      <Heading level={HeadingLevel.l5}>
        switch to{" "}
        <span
          className="text-pink hover:cursor-pointer"
          onClick={() => setLanguage(otherLanguage)}
        >
          {otherLanguage}
        </span>
      </Heading>
    </div>
    // <Select
    //   value={language}
    //   onValueChange={(value: "typescript" | "python") => setLanguage(value)}
    // >
    //   <SelectTrigger className="w-[180px]">
    //     <SelectValue placeholder="Select Language" />
    //   </SelectTrigger>
    //   <SelectContent>
    //     <SelectItem value="typescript">TypeScript</SelectItem>
    //     <SelectItem value="python">Python</SelectItem>
    //   </SelectContent>
    // </Select>
  );
}
