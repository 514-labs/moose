import React from "react";
import {
  Card,
  CardHeader,
  CardContent,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";

interface CommandMetadata {
  name: string;
  value: string | React.ReactNode;
}

interface CommandDocCardProps {
  title: string;
  description: string;
  className?: string;
}

export function CommandDocCard({
  title,
  description,
  className = "",
}: CommandDocCardProps) {
  // Example content for aurora init
  const command =
    "aurora init <project-name> <template-name> <--mcp <host>> <--location <location>>";
  const metadata: CommandMetadata[] = [
    {
      name: "Description",
      value:
        "Creates a data engineering project with Moose, with Aurora MCP preconfigured.",
    },
    { name: "<project-name>", value: "Name of your application (Required)" },
    {
      name: "<template-name>",
      value:
        "Template to base your app on (e.g. typescript-empty, ads-b) (Required)",
    },
    {
      name: "--mcp",
      value: "Which MCP host to use (Optional, e.g. cursor-project)",
    },
    { name: "--location", value: "Location of your app or service (Optional)" },
    {
      name: "--no-fail-already-exists",
      value: "Allow rerun if location exists (Optional)",
    },
  ];
  const [copied, setCopied] = React.useState(false);
  const handleCopy = () => {
    navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
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
              {metadata.map((meta) => (
                <tr
                  key={meta.name}
                  className="border-b last:border-0 align-top"
                >
                  <td className="py-1 pr-4 align-top whitespace-nowrap font-semibold">
                    {meta.name}
                  </td>
                  <td className="py-1 pr-4 align-top">{meta.value}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </CardContent>
    </Card>
  );
}
