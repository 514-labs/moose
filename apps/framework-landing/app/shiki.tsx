"use client";
import { codeToHtml } from "shiki";
import { useEffect, useState } from "react";

const mooseTheme = {
  name: "moose-theme",
  type: "dark",
  colors: {
    "editor.background": "#1E1E1E",
    "editor.foreground": "#E0E0E0",
  },
  tokenColors: [
    {
      scope: ["comment", "punctuation.definition.comment"],
      settings: {
        foreground: "#6A737D",
      },
    },
    {
      scope: ["string", "entity.name.section.markdown"],
      settings: {
        foreground: "#2CFF8D",
      },
    },
    {
      scope: ["constant", "constant.numeric", "constant.language"],
      settings: {
        foreground: "#00A3FF",
      },
    },
    {
      scope: ["variable", "variable.other"],
      settings: {
        foreground: "#D429FF",
      },
    },
    {
      scope: ["keyword", "storage.type", "storage.modifier"],
      settings: {
        foreground: "#641BFF",
      },
    },
    {
      scope: ["entity.name.function", "support.function"],
      settings: {
        foreground: "#1983FF",
      },
    },
    {
      scope: [
        "entity.name.type",
        "entity.name.class",
        "entity.other.inherited-class",
      ],
      settings: {
        foreground: "#FF2CC4",
      },
    },
    {
      scope: ["punctuation", "meta.brace"],
      settings: {
        foreground: "#E0E0E0",
      },
    },
    {
      scope: ["entity.name.tag", "meta.tag.sgml"],
      settings: {
        foreground: "#2CFF8D",
      },
    },
    {
      scope: ["markup.italic"],
      settings: {
        foreground: "#D429FF",
        fontStyle: "italic",
      },
    },
    {
      scope: ["markup.bold"],
      settings: {
        foreground: "#00A3FF",
        fontStyle: "bold",
      },
    },
    {
      scope: ["markup.heading"],
      settings: {
        foreground: "#FF2CC4",
        fontStyle: "bold",
      },
    },
  ],
};

interface CodeBlockProps {
  code: string;
  language: string;
  filename: string;
}

export default function CodeBlock({
  code,
  language,
  filename,
}: CodeBlockProps) {
  const [highlightedCode, setHighlightedCode] = useState("");

  useEffect(() => {
    async function highlight() {
      const highlighted = await codeToHtml(code, {
        lang: language,
        theme: mooseTheme,
      });
      setHighlightedCode(highlighted);
    }
    highlight();
  }, [code, language]);

  return (
    <div className="overflow-y-auto w-full h-[350px] bg-[#1E1E1E] rounded-lg">
      <div className="w-full text-sm text-primary bg-[#1A1A1A] p-3 border-b">
        {filename}
      </div>
      <div
        className="px-3 text-sm"
        dangerouslySetInnerHTML={{ __html: highlightedCode }}
      />
    </div>
  );
}
