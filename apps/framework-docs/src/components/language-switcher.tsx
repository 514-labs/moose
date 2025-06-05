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
    // <div className="my-5 flex flex-row w-full border-t border-b justify-between bg-black sticky top-20 z-99999999">
    //   <Heading level={HeadingLevel.l5} className="text-captialize">
    //     Viewing {language}
    //   </Heading>
    //   <Heading level={HeadingLevel.l5}>
    //     switch to{" "}
    //     <span
    //       className="text-moose-purple hover:cursor-pointer"
    //       onClick={() => setLanguage(otherLanguage)}
    //     >
    //       {otherLanguage}
    //     </span>
    //   </Heading>
    // </div>
    <div className="sticky top-[64px] z-30 py-2 px-4 border-t-0 bg-background border-y flex items-center justify-between">
      <Heading level={HeadingLevel.l5} className="text-capitalize">
        Viewing
      </Heading>
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
