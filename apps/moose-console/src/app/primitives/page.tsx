"use client";
import { useTrackPageView } from "app/trackable-components";
import { PrimitivesOverviewList } from "components/overview-list";
import { useContext } from "react";
import { VersionContext } from "version-context";

export default function PrimitivesPage() {
  useTrackPageView();
  const { models } = useContext(VersionContext);

  return (
    <section className="p-4 grow">
      <PrimitivesOverviewList models={models} />
    </section>
  );
}
