import React from "react";
import { Callout } from "./callout";
import Link from "next/link";
import { TypeScript, Python } from "./language-wrappers";

interface ExportRequirementProps {
  primitive: string;
  example?: string;
}

export function ExportRequirement({
  primitive,
  example,
}: ExportRequirementProps) {
  return (
    <Callout type="info" title="Export Required" compact>
      <TypeScript>
        <p>
          Ensure your {primitive.toLowerCase()} is correctly imported into your{" "}
          <code>app/index.ts</code> file.
        </p>
      </TypeScript>
      <Python>
        <p>
          Ensure your {primitive.toLowerCase()} is correctly imported into your{" "}
          <code>main.py</code> file.
        </p>
      </Python>
      <p>
        <Link
          href="/moose/local-dev#quick-example"
          className="text-blue-500 hover:underline"
        >
          Learn more about the export pattern →
        </Link>
      </p>
    </Callout>
  );
}
