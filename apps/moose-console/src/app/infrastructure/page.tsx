"use client";
import { InfrastructureOverviewList } from "components/overview-list";
import { useContext } from "react";
import { VersionContext } from "version-context";

export default function InfrastructurePage() {
  const { models } = useContext(VersionContext);

  return (
    <section className="p-4 grow">
      <InfrastructureOverviewList models={models} />
    </section>
  );
}
