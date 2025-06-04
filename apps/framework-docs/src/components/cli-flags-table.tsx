import React from "react";
import { Badge } from "@/components/ui/badge";
import { SmallTextEmbed } from "@/components/typography";
import { textBodyBase } from "@/components/typography";

export interface CliFlagArg {
  name: string; // e.g. "name" or "MCP-host"
  required: boolean;
  description: string;
  example?: string;
}

interface CliFlagsTableProps {
  items: CliFlagArg[];
}

export function CliFlagsTable({ items }: CliFlagsTableProps) {
  return (
    <div className="w-full">
      <div className="mb-2 text-sm text-muted-foreground font-normal">
        Arguments / Flags
      </div>
      <div className="flex flex-col gap-2 w-full">
        {items.map((item, idx) => (
          <div
            key={item.name + idx}
            className="flex flex-row items-start gap-4 border-b border-muted py-2 w-full"
          >
            <div
              className={`flex flex-row items-center min-w-[16rem] max-w-[16rem] ${textBodyBase}`}
            >
              <span
                className={`font-mono text-primary mr-2 truncate ${textBodyBase}`}
              >
                {item.name}
              </span>
              {item.required ?
                <Badge className="ml-1 bg-aurora-teal text-aurora-teal-foreground border-transparent">
                  required
                </Badge>
              : <Badge variant="secondary" className="ml-1">
                  optional
                </Badge>
              }
            </div>
            <div
              className={`flex-1 min-w-[20rem] max-w-[40rem] text-primary ${textBodyBase}`}
            >
              {item.description}
              {item.example && (
                <SmallTextEmbed className="ml-2 text-muted-foreground">
                  {item.example}
                </SmallTextEmbed>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
