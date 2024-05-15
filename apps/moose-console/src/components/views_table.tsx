"use client";
import { PreviewTable } from "./preview-table";
import { useRouter } from "next/navigation";
import { DataModel } from "app/types";

export function ViewsTable({ models }: { models: DataModel[] }) {
  const router = useRouter();

  const modelRows = models.map((model) => ({
    name: model.table?.name || "N/A",
    database: model.table?.database || "N/A",
    model: model.model.name,
    uuid: model.table?.uuid || "N/A",
  }));

  if (!modelRows.length) {
    return <div>No views found</div>;
  }
  return (
    <PreviewTable
      rows={modelRows}
      onRowClick={(row) => {
        if (row.uuid !== "N/A" && row.database !== "N/A") {
          router.push(
            `/infrastructure/databases/${row.database}/tables/${row.uuid}`,
          );
        }
      }}
    />
  );
}
