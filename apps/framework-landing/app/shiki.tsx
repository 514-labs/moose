"use client";
// import { codeToHtml } from "shiki";
import { useEffect, useState } from "react";
import { cn } from "@514labs/design-system-components/utils";

const mooseTheme = {
  name: "moose-theme",
  type: "dark",
  fg: "#E0E0E0",
  bg: "#000000",
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
        foreground: "#00A3FF",
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
        foreground: "#50FF76",
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
  className?: string;
}

async function shiki() {
  const { createHighlighter } = await import("shiki");
  return createHighlighter;
}

export default function CodeBlock({
  code,
  language,
  filename,
  className,
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
            bg: "#000000",
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
    <div
      className={cn(
        "overflow-y-scroll h-full w-full rounded-2xl border",
        className,
      )}
    >
      <div className="w-full text-sm text-muted-foreground p-3 font-mono">
        {filename}
      </div>
      <div
        className="flex-grow px-3 text-sm overflow-auto h-full" // Ensure it fills the height and scrolls
        dangerouslySetInnerHTML={{ __html: highlightedCode }}
      />
    </div>
  );
}
