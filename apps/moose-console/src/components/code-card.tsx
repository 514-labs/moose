"use client";

import { codeToHtml } from "shiki";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "components/ui/select";
import { Card, CardContent } from "./ui/card";
import { useEffect, useState } from "react";
import parse from "html-react-parser";
import { TrackButton } from "app/trackable-components";

interface Snippet {
  language: string;
  code: string;
}

type NonEmptyArray<T> = [T, ...T[]];

interface CodeCardProps {
  title: string;
  snippets: NonEmptyArray<Snippet>;
}

export default function CodeCard({ title, snippets }: CodeCardProps) {
  useEffect(() => {
    const formatedCode = async () => {
      const newFormatedCodeSnippets: { [key: string]: string } = {};

      for (const snippet of snippets) {
        const html = await codeToHtml(snippet.code, {
          lang: snippet.language,
          theme: "none",
        });
        newFormatedCodeSnippets[snippet.language] = html;
      }

      setFormatedCodeSnippets(newFormatedCodeSnippets);
    };
    formatedCode();
  }, [snippets]);

  const [selectedSnippet, setSelectedSnippet] = useState<Snippet>(snippets[0]);
  const [formatedCodeSnippets, setFormatedCodeSnippets] = useState<{
    [key: string]: string;
  }>({});

  return (
    <div>
      <div className="flex flex-row items-center py-2">
        <h2>{title}</h2>
        <span className="grow" />
        <TrackButton
          name="Copy Snippet"
          subject={selectedSnippet.code}
          variant="outline"
          className="mr-2"
          onClick={() => {
            navigator.clipboard.writeText(selectedSnippet.code);
          }}
        >
          copy
        </TrackButton>
        <div>
          <Select
            defaultValue={selectedSnippet.language}
            onValueChange={(value) => {
              const snippet = snippets.find(
                (snippet) => snippet.language === value
              );
              if (snippet) {
                setSelectedSnippet(snippet);
              }
            }}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {snippets.map((snippet, index) => (
                <SelectItem
                  key={index}
                  value={snippet.language}
                  onClick={() => setSelectedSnippet(snippet)}
                >
                  {snippet.language}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
      <Card className="rounded-2xl bg-muted ">
        <CardContent className="overflow-x-auto p-0 m-6">
          <code>
            {formatedCodeSnippets[selectedSnippet.language]
              ? parse(formatedCodeSnippets[selectedSnippet.language] as string)
              : "Loading..."}
          </code>
        </CardContent>
      </Card>
    </div>
  );
}
