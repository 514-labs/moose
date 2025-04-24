import React from "react";
import { useLanguage } from "./LanguageContext";

export function LanguageSwitcher() {
  const { language, setLanguage } = useLanguage();

  const handleChange = (event) => {
    setLanguage(event.target.value);
  };

  return (
    <div className="my-5 flex flex-row w-full border-t border-b justify-between py-5 sticky top-20 bg-black z-2000 ">
      <label htmlFor="language-select" className="font-semibold leading-6 py-2">
        Viewing
      </label>
      <select
        id="language-select"
        value={language}
        onChange={handleChange}
        className="border rounded p-3"
      >
        <option value="typescript">TypeScript</option>
        <option value="python">Python</option>
      </select>
    </div>
  );
}
