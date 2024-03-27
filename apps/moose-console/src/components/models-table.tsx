"use client";
import { DataModel, ModelMeta } from "app/db";
import { PreviewTable } from "./preview-table";
import { useRouter } from "next/navigation";

export function ModelsTable({ models }: { models: ModelMeta[] }) {
  const router = useRouter();
  const modelRows = models.map((model) => ({
    name: model.name,
    columns: model.columns.length,
    db_name: model.db_name,
  }));
  return (
    <PreviewTable
      rows={modelRows}
      onRowClick={(row) => router.push(`/primitives/models/${row.name}`)}
    />
  );
}
