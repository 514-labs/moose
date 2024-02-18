"use client";

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
        <Button
          variant="outline"
          onClick={() => {
            navigator.clipboard.writeText(code);
          }}
        >
          copy
        </Button>
      </h2>
      <Card className="rounded-2xl bg-muted p-4 overflow-x-auto flex flex-col">
        <code>{comment}</code>
        <code>{code}</code>
      </Card>
    </div>
  );
}
