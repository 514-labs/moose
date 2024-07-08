import { ColumnDef, SortingState } from "@tanstack/react-table";

export interface ParsedLogs {
  date: string;
  message: string;
  severityNumber: number;
  severityLevel: string;
  source: string;
  sessionId: string;
  serviceName: string;
  machineId: string;
  totalRowCount: number;
}

export type ParsedLogsResponse = {
  data: ParsedLogs[];
  meta: {
    totalRowCount: number;
  };
};

export async function fetchLogs({
  limit,
  offset,
  sorting,
  source,
  search,
}: {
  limit: number;
  offset: number;
  sorting: SortingState;
  search: string;
  source?: string;
}) {
  const url = new URL("http://localhost:4000/consumption/log_query");
  url.searchParams.append("limit", limit.toString());
  url.searchParams.append("offset", offset.toString());
  if (sorting.length > 0) {
    url.searchParams.append("sortCol", sorting[0].id);
    url.searchParams.append("sortDir", sorting[0].desc ? "DESC" : "ASC");
  }
  if (source) url.searchParams.append("source", source);
  if (search) url.searchParams.append("search", search);

  const response = await fetch(url.toString());
  const parsedLogs = await response.json();
  return parsedLogs as ParsedLogsResponse;
}

export function createLogColumns({
  selectedSource,
}: {
  selectedSource: string | undefined;
}): ColumnDef<ParsedLogs>[] {
  return [
    { accessorKey: "date", header: "Date" },
    {
      accessorKey: "severityLevel",
      header: "Severity",
      maxSize: 80,
    },
    {
      accessorKey: "machineId",
      header: "machineId",
    },
    {
      accessorKey: "source",
      header: "Source",
      minSize: 300,
      /*
      cell(props) {
        return (
          <span className="text-blue-500 underline">
            {selectedSource
              ? props.row.original.source.replace(selectedSource, "")
              : props.row.original.source}
          </span>
        );
      },
      */
    },
    {
      accessorKey: "message",
      header: "Message",
      minSize: 800,
    },
  ];
}

// Create a nested tree structure from a flat list of items
// using categories as the hierarchy, matching the above example
interface CategoryItem {
  categories: string[];
  occurences: number;
}

interface CategoryNode {
  id: string;
  name: string;
  children: CategoryNode[];
}

const addCategoryToTree = (
  tree: CategoryNode[],
  categories: string[],
  parentId: string = "", // Add a new parameter for parentId
): CategoryNode[] => {
  if (categories.length === 0) return tree;

  const [firstCategory, ...restCategories] = categories;
  const nodeId = parentId ? `${parentId}::${firstCategory}` : firstCategory; // Construct nodeId using parentId
  const existingNode = tree.find((n) => n.name === firstCategory);

  const updatedTree = existingNode
    ? tree
    : [...tree, { id: nodeId, name: firstCategory, children: [] }]; // Use nodeId for new nodes

  const updatedNode = {
    ...(existingNode ?? { id: nodeId, name: firstCategory, children: [] }),
    children: addCategoryToTree(
      existingNode ? existingNode.children : [],
      restCategories,
      nodeId,
    ), // Pass nodeId as parentId for recursive calls
  };

  return updatedTree.map((n) => (n.name === firstCategory ? updatedNode : n));
};

function rollUpCategories(items: CategoryItem[]): CategoryNode[] {
  return items.reduce(
    (tree: CategoryNode[], item) => addCategoryToTree(tree, item.categories),
    [],
  );
}

export async function fetchLogHierarchy() {
  const url = new URL("http://localhost:4000/consumption/log_hierarchy");
  const response = await fetch(url.toString());
  const occurences = (await response.json()) as CategoryItem[];
  return rollUpCategories(occurences);
}
