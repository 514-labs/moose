"use client";
import React from "react";
import { Tree, File, Folder, CollapseButton } from "./tree-view";
import { CategoryNode } from "@/lib/data-fetch";
import { SeverityLevel, severityLevelColors } from "@/lib/utils";
import { Badge } from "./badge";

type TOCProps = {
  toc: CategoryNode[];
  selectedId: string | undefined;
  setSelectedId: React.Dispatch<React.SetStateAction<string | undefined>>;
};

export default function Hierarchy({
  toc,
  selectedId,
  setSelectedId,
}: TOCProps) {
  return (
    <Tree
      selectedId={selectedId}
      setSelectedId={setSelectedId}
      className="w-full h-full bg-background p-2 rounded-md"
      indicator={true}
    >
      {toc.map((element, _) => (
        <TreeItem key={element.id} elements={[element]} />
      ))}
      <CollapseButton elements={toc} expandAll={true} />
    </Tree>
  );
}

type TreeItemProps = {
  elements: CategoryNode[];
};

export const TreeItem = ({ elements }: TreeItemProps) => {
  return (
    <ul className="w-full space-y-1">
      {elements.map((element) => (
        <li key={element.id} className="w-full space-y-2">
          {element.children && element.children?.length > 0 ? (
            <Folder
              element={element.name}
              value={element.id}
              isSelectable={true}
              className="px-px pr-1"
              inline={<MetricChips metrics={element.metrics} />}
            >
              <TreeItem
                key={element.id}
                aria-label={`folder ${element.name}`}
                elements={element.children}
              />
            </Folder>
          ) : (
            <File key={element.id} value={element.id} isSelectable={true}>
              <span>{element?.name}</span>
              <MetricChips metrics={element.metrics} />
            </File>
          )}
        </li>
      ))}
    </ul>
  );
};

function MetricChip({ name, count }: { name: SeverityLevel; count: number }) {
  if (count === 0) return null;
  return <Badge className={severityLevelColors[name]}>{count}</Badge>;
}

function MetricChips({ metrics }: { metrics: CategoryNode["metrics"] }) {
  return (
    <div className="ml-auto flex gap-x-1">
      <MetricChip name={SeverityLevel.ERROR} count={metrics.error} />
      <MetricChip name={SeverityLevel.WARN} count={metrics.warn} />
      <MetricChip name={SeverityLevel.DEBUG} count={metrics.debug} />
      <MetricChip name={SeverityLevel.INFO} count={metrics.info} />
    </div>
  );
}
