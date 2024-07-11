import { ColumnDef, SortingState } from "@tanstack/react-table";
import { SeverityLevel, getApiRoute } from "./utils";

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
  severity,
}: {
  limit: number;
  offset: number;
  sorting: SortingState;
  search: string;
  source?: string;
  severity: SeverityLevel[];
}) {
  const url = new URL(`${getApiRoute()}/consumption/log_query`);
  url.searchParams.append("limit", limit.toString());
  url.searchParams.append("offset", offset.toString());
  if (sorting.length > 0) {
    url.searchParams.append("sortCol", sorting[0].id);
    url.searchParams.append("sortDir", sorting[0].desc ? "DESC" : "ASC");
  }
  if (source) url.searchParams.append("source", source);
  if (search) url.searchParams.append("search", search);
  if (severity.length > 0)
    url.searchParams.append("severity", severity.join(","));

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
    },
    {
      accessorKey: "message",
      header: "Message",
      size: 500,
    },
  ];
}

// Create a nested tree structure from a flat list of items
// using categories as the hierarchy, matching the above example
interface RawCategoryItem {
  categories: string[];
  info: string;
  warn: string;
  error: string;
  debug: string;
}

interface CategoryItem {
  categories: string[];
  metrics: {
    info: number;
    warn: number;
    error: number;
    debug: number;
  };
}

export interface CategoryNode {
  id: string;
  name: string;
  children: CategoryNode[];
  metrics: {
    info: number;
    warn: number;
    error: number;
    debug: number;
  };
}

function addMetrics(
  one: CategoryNode["metrics"],
  two: CategoryNode["metrics"],
) {
  return {
    info: one.info + two.info,
    debug: one.debug + two.debug,
    warn: one.debug + two.debug,
    error: one.debug + two.debug,
  };
}

const addCategoryToTree = (
  tree: CategoryNode[],
  categories: string[],
  parentId: string = "", // Keep the new parameter for parentId
  metrics: CategoryItem["metrics"] = { info: 0, warn: 0, error: 0, debug: 0 }, // Add a parameter for occurrences
): CategoryNode[] => {
  if (categories.length === 0) return tree;

  const [firstCategory, ...restCategories] = categories;
  const nodeId = parentId ? `${parentId}::${firstCategory}` : firstCategory; // Keep constructing nodeId using parentId
  const existingNode = tree.find((n) => n.name === firstCategory);

  // Update or create the node with the new occurrences count
  const updatedNode = existingNode
    ? { ...existingNode, metrics: addMetrics(existingNode.metrics, metrics) }
    : { id: nodeId, name: firstCategory, children: [], metrics };

  const updatedTree = existingNode ? tree : [...tree, updatedNode]; // Add updatedNode to the tree if it's new

  // Recursively update or add children with the correct occurrences count
  updatedNode.children = addCategoryToTree(
    existingNode ? existingNode.children : [],
    restCategories,
    nodeId,
    metrics, // Pass occurrences through recursive calls
  );

  // Only map over the tree if the node existed, otherwise, just return the updated tree
  return existingNode
    ? tree.map((n) => (n.name === firstCategory ? updatedNode : n))
    : updatedTree;
};

function rollUpCategories(items: CategoryItem[]): CategoryNode[] {
  // Adjust the initial call to include occurrences if needed
  return items.reduce(
    (tree: CategoryNode[], item) =>
      addCategoryToTree(tree, item.categories, "", item.metrics), // Start with 1 occurrence for each top-level category
    [],
  );
}

export async function fetchLogHierarchy({
  source,
  search,
  severity,
}: {
  search: string;
  source?: string;
  severity: SeverityLevel[];
}) {
  const url = new URL(`${getApiRoute()}/consumption/log_hierarchy`);
  if (source) url.searchParams.append("source", source);
  if (search) url.searchParams.append("search", search);
  if (severity.length > 0)
    url.searchParams.append("severity", severity.join(","));

  const response = await fetch(url.toString());
  const occurences = (await response.json()) as RawCategoryItem[];
  const processed = occurences.map((occ) => {
    const { categories, ...metrics } = occ;
    return {
      categories,
      metrics: {
        info: parseInt(metrics.info),
        debug: parseInt(metrics.debug),
        warn: parseInt(metrics.warn),
        error: parseInt(metrics.error),
      },
    };
  });
  return rollUpCategories(processed);
}
