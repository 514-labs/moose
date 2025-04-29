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
    <div className="my-5 flex flex-row w-full border-t border-b justify-between bg-black sticky top-20 z-99999">
      <Heading level={HeadingLevel.l5} className="text-captialize">
        Viewing {language}
      </Heading>
      <Heading level={HeadingLevel.l5}>
        switch to{" "}
        <span
          className="text-moose-purple hover:cursor-pointer"
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
