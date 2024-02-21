/* eslint-disable turbo/no-undeclared-env-vars */

import Link from "next/link";
import { Project, Table, getCliData } from "app/db";
import { unstable_noStore as noStore } from "next/cache";
import TableTabs from "./table-tabs";
import { getClient } from "lib/clickhouse";
import {
  bashSnippet,
  clickhouseJSSnippet,
  clickhousePythonSnippet,
  jsSnippet,
  pythonSnippet,
} from "lib/snippets";
import { getModelFromTable } from "lib/utils";

async function _describeTable(
  databaseName: string,
  tableName: string,
  project: Project,
): Promise<any> {
  const client = getClient(project);

  const resultSet = await client.query({
    query: `DESCRIBE TABLE ${databaseName}.${tableName}`,
    format: "JSONEachRow",
  });

  return resultSet.json();
}

const isView = (table: Table) => table.engine === "MaterializedView";

export default async function Page({
  params,
}: {
  params: { databaseName: string; tableId: string };
  searchParams: { tab: string };
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();

  const table = data.tables.find((table) => table.uuid === params.tableId);
  const model = getModelFromTable(table, data);

  return (
    <section className="p-4 max-h-screen flex-grow overflow-y-auto flex flex-col">
      <div className="py-10">
        <div className="text-6xl">
          <Link className="text-muted-foreground" href={"/"}>
            ../
          </Link>
          <Link
            className="text-muted-foreground"
            href={`/infrastructure/databases/tables?type=${isView(table) ? "view" : "table"}`}
          >
            {isView(table) ? "views" : "tables"}/
          </Link>
          {table.name}
        </div>
        <div className="text-muted-foreground">{table.engine}</div>
      </div>
      <div className="space-x-3 flex-grow">
        <TableTabs
          table={table}
          cliData={data}
          bashSnippet={bashSnippet(data, model)}
          jsSnippet={jsSnippet(data, model)}
          pythonSnippet={pythonSnippet(data, model)}
          clickhouseJSSnippet={clickhouseJSSnippet(data, model)}
          clickhousePythonSnippet={clickhousePythonSnippet(data, model)}
        />
      </div>
    </section>
  );
}
