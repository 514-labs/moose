"use client";

import { ColumnDef } from "@tanstack/react-table";
// This type is used to define the shape of our data.
// You can use a Zod schema here if you want.

export function createColumns<T extends {}>(object: T): ColumnDef<T>[] {
  return Object.keys(object).map((key) => ({ accessorKey: key, header: key }));
}
