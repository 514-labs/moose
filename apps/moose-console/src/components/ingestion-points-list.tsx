"use client";
import { useRouter } from "next/navigation";
import { PreviewTable } from "./preview-table";
import { Route } from "app/types";

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

  if (!ingestionRows.length) {
    return <div>No ingestion points found</div>;
  }

  return (
    <PreviewTable
      rows={ingestionRows}
      onRowClick={(point) =>
        router.push(
          `/infrastructure/ingestion-points/${point.route.split("/").at(-2)}`,
        )
      }
    ></PreviewTable>
  );
}
