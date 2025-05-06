import React from "react";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useLanguage } from "./LanguageContext";
import { useLanguage } from "./LanguageContext";

export function LanguageSwitcher() {
  const { language, setLanguage } = useLanguage();

  const handleChange = (value: "typescript" | "python") => {
    setLanguage(value);
  };
  const handleChange = (value: "typescript" | "python") => {
    setLanguage(value);
  };

  return (
    <div className="sticky top-10 bg-white dark:bg-black z-10 py-10 pb-5 flex flex-row w-full border-b justify-between">
      <label htmlFor="language-select" className="font-semibold leading-10">
        Viewing
      </label>
      <Select onValueChange={handleChange} value={language}>
        <SelectTrigger
          id="language-select"
          className="border rounded p-2 text-moose-white w-auto"
        >
          <SelectValue placeholder="TypeScript" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="typescript">TypeScript</SelectItem>
          <SelectItem value="python">Python</SelectItem>
        </SelectContent>
      </Select>
    <div className="sticky top-10 bg-white dark:bg-black z-10 py-10 pb-5 flex flex-row w-full border-b justify-between">
      <label htmlFor="language-select" className="font-semibold leading-10">
        Viewing
      </label>
      <Select onValueChange={handleChange} value={language}>
        <SelectTrigger
          id="language-select"
          className="border rounded p-2 text-moose-white w-auto"
        >
          <SelectValue placeholder="TypeScript" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="typescript">TypeScript</SelectItem>
          <SelectItem value="python">Python</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}

