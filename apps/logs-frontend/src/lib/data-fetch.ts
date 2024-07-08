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
}: {
  limit: number;
  offset: number;
  sorting: SortingState;
}) {
  const url = new URL("http://localhost:4000/consumption/log_query");
  url.searchParams.append("limit", limit.toString());
  url.searchParams.append("offset", offset.toString());
  if (sorting.length > 0) {
    url.searchParams.append("sortCol", sorting[0].id);
    url.searchParams.append("sortDir", sorting[0].desc ? "DESC" : "ASC");
  }
  const response = await fetch(url.toString());
  const parsedLogs = await response.json();
  return parsedLogs as ParsedLogsResponse;
}

export const logColumns: ColumnDef<ParsedLogs>[] = [
  { accessorKey: "date", header: "Date" },
  {
    accessorKey: "severityNumber",
    header: "Severity Number",
  },
  {
    accessorKey: "severityLevel",
    header: "Severity Level",
  },
  {
    accessorKey: "source",
    header: "Source",
  },
  {
    accessorKey: "message",
    header: "Message",
    maxSize: 300,
  },
  {
    accessorKey: "serviceName",
    header: "Service Name",
  },
  {
    accessorKey: "machineId",
    header: "machineId",
  },
];
