"use client";

import { TrackButton } from "app/trackable-components";
import { Button } from "./ui/button";
import { Card } from "./ui/card";

interface SnippetCardProps {
  title: string;
  code: string;
  comment?: string;
}

export default function SnippetCard({
  title,
  code,
  comment,
}: SnippetCardProps) {
  return (
    <div className="py-4">
      <h2 className="py-2 flex flex-row items-center">
        <span>{title}</span>
        <span className="grow" />
        <TrackButton
          name="Copy Snippet"
          subject={code}
          variant="outline"
          onClick={() => {
            navigator.clipboard.writeText(code);
          }}
        >
          copy
        </TrackButton>
      </h2>
      <Card className="rounded-2xl bg-muted p-4 overflow-x-auto flex flex-col">
        <code>{comment}</code>
        <code>{code}</code>
      </Card>
    </div>
  );
}
