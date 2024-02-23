"use client";
import { Route } from "app/db";
import { useRouter } from "next/navigation";
import { PreviewTable } from "./preview-table";

interface IngestionPointsListProps {
  ingestionPoints: Route[];
}

export function IngestionPointsList({
  ingestionPoints,
}: IngestionPointsListProps) {
  const router = useRouter();
  const ingestionRows = ingestionPoints.map((points) => ({
    route: points.route_path,
    table_name: points.table_name,
  }));

  return (
    <PreviewTable
      rows={ingestionRows}
      onRowClick={(point) =>
        router.push(`/infrastructure/ingestion-points/${point.route.split("/").at(-1)}`)
      }
    ></PreviewTable>
  );
}
