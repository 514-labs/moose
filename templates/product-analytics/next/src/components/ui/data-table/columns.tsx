"use client";

import { ColumnDef } from "@tanstack/react-table";
import { Button } from "../button";
import { ArrowUp, ArrowDown } from "lucide-react";

function SortIcon({ direction }: { direction: "asc" | "desc" | false }) {
  if (!direction) return null;
  return direction === "asc" ? <ArrowDown /> : <ArrowUp />;
}

export function createColumns<T extends {}>(object: T): ColumnDef<T>[] {
  return Object.keys(object).map((key) => ({
    accessorKey: key,
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          {column.id}
          <SortIcon direction={column.getIsSorted()} />
        </Button>
      );
    },
  }));
}
