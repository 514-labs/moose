import React from "react";

import { unstable_noStore as noStore } from "next/cache";
import { Separator } from "components/ui/separator";
import { NavBreadCrumb } from "components/nav-breadcrumb";
import { ViewsTable } from "components/views_table";
import { Table, getCliData } from "app/db";

type View = "view" | "table";

function getTablesForView(tables: Table[], param: { type: View }) {
  const cleanedTables = tables.filter((t) => !t.name.includes(".inner"));

  switch (param.type) {
    case "view":
      return cleanedTables.filter((t) => t.engine === "MaterializedView");
    case "table":
      return cleanedTables.filter((t) => t.engine !== "MaterializedView");
    default:
      return cleanedTables;
  }
}
async function TablesPage({ searchParams }: any) {
  noStore();
  const data = await getCliData();

  const tables = getTablesForView(data.tables, searchParams).filter(
    (table) => table.engine !== "Kafka"
  );

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
      <NavBreadCrumb />
      <div className="py-10">
        <div className="text-8xl">
          {searchParams.type === "view" ? "Views" : "Tables"}
        </div>
      </div>
      <div className="">
        <Separator />
        <ViewsTable tables={tables} data={data} />
      </div>
    </section>
  );
}

export default TablesPage;
