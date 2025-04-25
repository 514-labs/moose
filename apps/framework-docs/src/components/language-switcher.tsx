import React from "react";
import { useLanguage } from "./LanguageContext";

export function LanguageSwitcher() {
  const { language, setLanguage } = useLanguage();

  const handleChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setLanguage(event.target.value as "typescript" | "python");
  };

  return (
    <div className="sticky top-20 bg-white dark:bg-black z-10 py-5 my-5 flex flex-row w-full border-t border-b justify-between">
      <label htmlFor="language-select" className="font-semibold leading-10">
        Viewing
      </label>
      <select
        id="language-select"
        value={language}
        onChange={handleChange}
        className="border rounded p-2 text-moose-white"
      >
        <option value="typescript">TypeScript</option>
        <option value="python">Python</option>
      </select>
    </div>
  );
}
