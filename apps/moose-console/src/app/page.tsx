"use client";
import {
  InfrastructureOverviewList,
  PrimitivesOverviewList,
} from "components/overview-list";
import { useContext } from "react";
import { VersionContext } from "version-context";

export default function OverviewPage() {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  const { models } = useContext(VersionContext);

  return (
    <section className="p-4 grow overflow-y-scroll">
      <PrimitivesOverviewList models={models} />
      <InfrastructureOverviewList models={models} />
    </section>
  );
}
