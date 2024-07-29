"use client";
// import { codeToHtml } from "shiki";
import { useEffect, useState } from "react";

const mooseTheme = {
  name: "moose-theme",
  type: "dark",
  fg: "#E0E0E0",
  bg: "#1E1E1E",
  settings: [
    {
      scope: ["comment", "punctuation.definition.comment"],
      settings: {
        foreground: "#00A4C8",
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
        foreground: "#00A4C8",
      },
    },
    {
      scope: ["entity.name.function", "support.function"],
      settings: {
        foreground: "#2CFF8D",
      },
    },
    {
      scope: [
        "entity.name.type",
        "entity.name.class",
        "entity.other.inherited-class",
      ],
      settings: {
        foreground: "#00A3FF",
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

async function shiki() {
  const { createHighlighter } = await import("shiki");
  return createHighlighter;
}

export default function CodeBlock({
  code,
  language,
  filename,
}: CodeBlockProps) {
  const [highlightedCode, setHighlightedCode] = useState("");

  useEffect(() => {
    async function highlight() {
      const createHighlighter = await shiki();
      const highlighter = await createHighlighter({
        themes: [
          {
            name: "moose-theme",
            fg: "#E0E0E0",
            bg: "#1E1E1E",
            settings: mooseTheme.settings,
          },
        ],
        langs: ["typescript", "python"],
      });

      const highlighted = highlighter.codeToHtml(code, {
        lang: language,
        theme: "moose-theme",
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
