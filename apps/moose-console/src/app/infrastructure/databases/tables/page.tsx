import React from "react";
import { Table, getCliData } from "app/db";
import { unstable_noStore as noStore } from "next/cache";
import Link from "next/link";
import { Separator } from "components/ui/separator";

type View = "view" | "table";

function getTablesForView(tables: Table[], view: View) {
  const cleanedTables = tables.filter((t) => !t.name.includes(".inner"));

  switch (view) {
    case "view":
      return cleanedTables.filter((t) => t.engine === "MaterializedView");
    case "table":
      return cleanedTables.filter((t) => t.engine !== "MaterializedView");
    default:
      return cleanedTables;
  }
}
async function TablesPage({ searchParams }) {
  noStore();
  const data = await getCliData();

  const tables = getTablesForView(data.tables, searchParams);

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
      <div className="py-10">
        <div className="text-6xl">
          <Link className="text-muted-foreground" href={"/"}>
            overview/
          </Link>
          {searchParams.type === "view" ? "Views" : "Tables"}
        </div>
      </div>
      <div className="">
        <Separator />
        {tables.map((table, index) => (
          <Link
            key={index}
            href={`/infrastructure/databases/${table.database}/tables/${table.uuid}`}
          >
            <div
              key={index}
              className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer"
            >
              <div className="py-2 flex flex-row">
                <div>
                  <div>{table.name}</div>
                  <div className="text-muted-foreground">{table.database}</div>
                </div>
                <span className="flex-grow" />
              </div>
              <Separator />
            </div>
          </Link>
        ))}
      </div>
    </section>
  );
}

export default TablesPage;
