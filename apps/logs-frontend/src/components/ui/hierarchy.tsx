"use client";
import React from "react";
import {
  Tree,
  TreeViewElement,
  File,
  Folder,
  CollapseButton,
} from "./tree-view";

type TOCProps = {
  toc: TreeViewElement[];
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
  elements: TreeViewElement[];
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
            </File>
          )}
        </li>
      ))}
    </ul>
  );
};
