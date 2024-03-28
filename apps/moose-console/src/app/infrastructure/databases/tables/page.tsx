"use client";
import React, { useContext } from "react";

import { Separator } from "components/ui/separator";
import { NavBreadCrumb } from "components/nav-breadcrumb";
import { ViewsTable } from "components/views_table";
import { tableIsView } from "lib/utils";
import { VersionContext } from "version-context";

export default function TablesPage({
  searchParams,
}: {
  searchParams: { type: string };
}) {
  const { models } = useContext(VersionContext);

  const isView = searchParams.type == "view";

  const filteredModels = models.filter(
    (model) => tableIsView(model.table) === isView,
  );

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
      <NavBreadCrumb />
      <div className="py-10">
        <div className="text-8xl">{isView ? "Views" : "Tables"}</div>
      </div>
      <div className="">
        <Separator />
        <ViewsTable models={filteredModels} />
      </div>
    </section>
  );
}
