import React from "react";
import {
  Card,
  CardHeader,
  CardContent,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";

interface CommandMetadata {
  name: string;
  value?: string | React.ReactNode;
  description?: string;
  required?: boolean;
  example?: string;
}

interface CommandDocCardProps {
  title: string;
  description: string;
  command: string;
  parameters: CommandMetadata[];
  className?: string;
  /**
   * Semantic heading level for the title (e.g., 'h2', 'h3', 'h4').
   * Defaults to 'h3'. Used for TOC integration.
   */
  headingLevel?: "h2" | "h3" | "h4" | "h5";
}

export function CommandDocCard({
  title,
  description,
  command,
  parameters = [],
  className = "",
  headingLevel = "h3",
}: CommandDocCardProps) {
  const [copied, setCopied] = React.useState(false);
  const handleCopy = () => {
    navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  const HeadingTag = headingLevel;
  const headingId = title.toLowerCase().replace(/\s+/g, "-");

  return (
    <Card className={className}>
      <CardHeader>
        <HeadingTag
          id={headingId}
          className="scroll-mt-24 text-lg font-semibold tracking-tight flex items-center gap-2"
        >
          <a href={`#${headingId}`} className="hover:underline text-inherit">
            {title}
          </a>
        </HeadingTag>
      </CardHeader>
      <CardContent>
        {description && (
          <div className="mb-4 text-muted-foreground text-sm leading-relaxed">
            {description}
          </div>
        )}
        <div className="flex items-center gap-2 mb-4">
          <pre className="bg-muted rounded px-3 py-2 text-sm overflow-x-auto flex-1">
            {command}
          </pre>
          <Button size="sm" variant="outline" onClick={handleCopy}>
            {copied ? "Copied!" : "Copy"}
          </Button>
        </div>
        <div className="w-full overflow-x-auto">
          <table className="w-full text-sm border-collapse">
            <tbody>
              {parameters
                .filter((meta) => meta.name !== "Description")
                .map((meta) => (
                  <tr
                    key={meta.name}
                    className="border-b last:border-0 align-top"
                  >
                    <td className="py-1 pr-4 align-top whitespace-nowrap font-semibold">
                      <code className="px-1.5 py-0.5 rounded bg-muted text-primary font-mono text-xs">
                        {meta.name}
                      </code>
                    </td>
                    <td className="py-1 pr-4 align-top">
                      {meta.description || meta.value}{" "}
                      <span
                        className={
                          meta.required ?
                            "ml-2 inline-block px-2 py-0.5 rounded-full bg-green-100 text-green-800 text-xs font-medium"
                          : "ml-2 inline-block px-2 py-0.5 rounded-full bg-gray-100 text-gray-800 text-xs font-medium"
                        }
                      >
                        {meta.required ? "Required" : "Optional"}
                      </span>
                    </td>
                    <td className="py-1 pr-4 align-top">
                      {meta.example ? `e.g. ${meta.example}` : ""}
                    </td>
                  </tr>
                ))}
            </tbody>
          </table>
        </div>
      </CardContent>
    </Card>
  );
}
