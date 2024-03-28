"use client";

import { IngestionPointsList } from "components/ingestion-points-list";
import { NavBreadCrumb } from "components/nav-breadcrumb";
import { useContext } from "react";
import { VersionContext } from "version-context";

export default function IngestionPointsPage() {
  const { models } = useContext(VersionContext);
  const ingestionPoints = models.map(({ ingestion_point }) => ingestion_point);

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
      <NavBreadCrumb />
      <div className="py-10">
        <div className="text-8xl">
          {ingestionPoints.length} Ingestion Points
        </div>
      </div>
      <IngestionPointsList ingestionPoints={ingestionPoints} />
    </section>
  );
}
