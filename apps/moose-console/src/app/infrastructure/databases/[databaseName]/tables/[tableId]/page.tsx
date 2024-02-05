/* eslint-disable turbo/no-undeclared-env-vars */

import {  createClient } from "@clickhouse/client-web";
import { Field } from "app/mock";
import Link from "next/link";
import { Table, getCliData } from "app/db";
import { unstable_noStore as noStore } from "next/cache";
import TableTabs from "./table-tabs";
import { clickhouseJSSnippet, clickhousePythonSnippet, jsSnippet, pythonSnippet } from "lib/snippets";
import { getModelFromTable, getRelatedInfra } from "lib/utils";


function getClient() {
  const CLICKHOUSE_HOST = process.env.CLICKHOUSE_HOST || "localhost";
  // Environment variables are always strings
  const CLICKHOUSE_PORT = process.env.CLICKHOUSE_PORT || "18123";
  const CLICKHOUSE_USERNAME = process.env.CLICKHOUSE_USERNAME || "panda";
  const CLICKHOUSE_PASSWORD = process.env.CLICKHOUSE_PASSWORD || "pandapass";
  const CLICKHOUSE_DB = "local";

  const client = createClient({
    host: `http://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
    username: CLICKHOUSE_USERNAME,
    password: CLICKHOUSE_PASSWORD,
    database: CLICKHOUSE_DB,
  });

  return client;
}


async function describeTable(databaseName: string, tableName: string): Promise<any> {
  const client = getClient();

  const resultSet = await client.query({
    query: `DESCRIBE TABLE ${databaseName}.${tableName}`,
    format: "JSONEachRow",
  });

  return resultSet.json();

}

interface FieldsListCardProps {
  fields: Field[]
}



const isView = (table: Table) => table.engine === "MaterializedView";

export default async function Page({
  params,
}: {
  params: {databaseName: string,  tableId: string };
  searchParams: { tab: string};
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();

  const table = data.tables.find((table) => table.uuid === params.tableId);
  const tableMeta = await describeTable(params.databaseName, table.name);
  const model = getModelFromTable(table, data);
  const infra = getRelatedInfra(model, data, table)

  return (
    <section className="p-4 max-h-screen flex-grow overflow-y-auto flex flex-col">
        <div className="py-10">
          <div className="text-6xl">
            <Link className="text-muted-foreground" href={"/"}>../</Link>
            <Link className="text-muted-foreground" href={`/infrastructure/databases/tables?type=${isView(table) ? "view" : "table"}`}>{isView(table) ? "views" : "tables"}/</Link>
            {table.name}
          </div>
          <div className="text-muted-foreground">{table.engine}</div>
        </div>
        <div className="space-x-3 flex-grow">
          <TableTabs table={table} cliData={data} jsSnippet={jsSnippet(data, model)} pythonSnippet={pythonSnippet(data, model)} clickhouseJSSnippet={clickhouseJSSnippet(data, model)} clickhousePythonSnippet={clickhousePythonSnippet(data, model)}/>
        </div>
      </section>  
  );
}
