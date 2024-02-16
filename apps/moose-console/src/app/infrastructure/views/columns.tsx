"use client";

import { Button } from "components/ui/button";
import { View } from "../mock";
import { ColumnDef } from "@tanstack/react-table";
import { ArrowUpDown, ChevronRight } from "lucide-react";
import Link from "next/link";

export const viewColumns: ColumnDef<View>[] = [
  { accessorKey: "id", header: "ID" },
  { accessorKey: "databaseId", header: "Database" },
  { accessorKey: "parentTable", header: "Parent Table ID" },
  { accessorKey: "name", header: "Name" },
  { accessorKey: "description", header: "Description" },
  {
    accessorKey: "status",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Status
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
  },
  { accessorKey: "docLink", header: "Documentation" },
  { accessorKey: "version", header: "Version" },
  { accessorKey: "consolePath", header: "Console Path" },
  { accessorKey: "fields", header: "Fields" },
  { accessorKey: "fieldCount", header: "Field Count" },
  { accessorKey: "rowCount", header: "Row Count" },
  { accessorKey: "lastUpdated", header: "Last Updated" },
  { accessorKey: "lastUpdatedBy", header: "Last Updated By" },
  { accessorKey: "samples", header: "Samples" },
  { accessorKey: "modelId", header: "Model Id" },
  { accessorKey: "errors", header: "Errors" },
  { accessorKey: "environment", header: "Environment" },
  {
    id: "actions",
    cell: ({ row }) => {
      const _view = row.original;

      return (
        <Link href={""}>
          <Button variant="ghost" className="h-8 w-8 p-0">
            <span className="sr-only">Open menu</span>
            <ChevronRight className="h-4 w-4" />
          </Button>
        </Link>
      );
    },
  },
];
