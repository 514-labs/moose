"use client";
import { CliData, Table } from "app/db";
import { PreviewTable } from "./preview-table";
import { useRouter } from "next/navigation";
import { getModelFromTable } from "lib/utils";

export function ViewsTable({
  tables,
  data,
}: {
  tables: Table[];
  data: CliData;
}) {
  const router = useRouter();
  const modelRows = tables.map((table) => ({
    name: table.name,
    database: table.database,
    model: getModelFromTable(table, data).name,
    uuid: table.uuid
  }));
  return (
    <PreviewTable
      rows={modelRows}
      onRowClick={(row) =>
        router.push(`/infrastructure/databases/${row.database}/tables/${row.uuid}`)
      }
    />
  );
}
