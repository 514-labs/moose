import React from "react";
import { Badge } from "./ui/badge";
import { SmallTextEmbed } from "./typography";

export interface ArgTableRow {
  name: string;
  required?: boolean;
  description: string;
  examples?: string[];
}

export interface ArgTableProps {
  args: ArgTableRow[];
}

export const ArgTable: React.FC<ArgTableProps> = ({ args }) => {
  return (
    <div className="mt-6 divide-y divide-border border rounded-lg overflow-hidden">
      {args.map((arg, idx) => (
        <div
          key={arg.name + idx}
          className="flex flex-row items-start px-4 py-3 gap-4"
        >
          <div className="w-48 shrink-0 font-mono text-sm text-primary/90 pt-1">
            {arg.name}
          </div>
          <div className="flex-1 flex flex-col gap-1">
            <span className="text-primary text-sm align-middle">
              {arg.required && (
                <span className="align-middle mr-2 inline-block">
                  <Badge variant="default">Required</Badge>
                </span>
              )}
              {arg.description}
              {arg.examples && arg.examples.length > 0 && (
                <span className="text-muted-foreground">
                  {" "}
                  &ndash; {arg.examples.join(", ")}
                </span>
              )}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
};

export default ArgTable;
