/* eslint-disable turbo/no-undeclared-env-vars */

import {  createClient } from "@clickhouse/client-web";
import { Field } from "app/mock";
import { PreviewTable } from "components/preview-table";
import QueryInterface from "components/query-interface";
import { tabListStyle, tabTriggerStyle } from "components/style-utils";
import { Card, CardContent } from "components/ui/card";
import { Separator } from "components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";
import { cn } from "lib/utils";
import Link from "next/link";

import { Table, getCliData } from "app/db";
import { unstable_noStore as noStore } from "next/cache";
import { Button } from "components/ui/button";


function getClient() {
  const CLICKHOUSE_HOST = process.env.CLICKHOUSE_HOST || "localhost";
  // Environment variables are always strings
  const CLICKHOUSE_PORT = process.env.CLICKHOUSE_PORT || "18123";
  const CLICKHOUSE_USERNAME = process.env.CLICKHOUSE_USERNAME || "panda";
  const CLICKHOUSE_PASSWORD = process.env.CLICKHOUSE_PASSWORD || "pandapass";
  const CLICKHOUSE_DB = "default";

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


async function getTable(databaseName: string, tableName: string): Promise<any> {
  const client = getClient();

  try {
    const resultSet = await client.query({
      query: `SELECT * FROM ${databaseName}.${tableName} LIMIT 50`,
      format: "JSONEachRow",
    });

    return resultSet.json();
  } catch (error) {
    return error
  }
}


interface FieldsListCardProps {
  fields: Field[]
}


function FieldsList({ fields }: FieldsListCardProps) {
  return (
      <div>
          {fields.map((field, index) => (
              <div key={index} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="py-4 flex flex-row ">
                      <div>
                          <div className="text-xl">{field.name}</div>
                          <div className="text-muted-foreground">{field.type}</div>
                      </div>
                      <span className="flex-grow"/>
                  </div>
                  <Separator/>
              </div>
          ))}
      </div>                
  )
}

function ClickhouseTableRestriction({viewId, databaseName}) {

  return (
    <div className="py-4">
      <div className="text-muted-foreground max-w-xl">
        This table is an ingestion Clickhouse table. You unfortunately query it directly. To get a preview of the data, head to its associated view 
      </div>
      <Link className=" underline" href={`/infrastructure/databases/${databaseName}/tables/${viewId}`}>
        <Button variant="default" className="mt-4">
          go to view
        </Button>
      </Link>
    </div>
  )
}

const isView = (table: Table) => table.engine === "MaterializedView";

export default async function Page({
  params,
}: {
  params: {databaseName: string,  tableId: string };
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();

  const table = data.tables.find((table) => table.uuid === params.tableId);

  const associated_view = data.tables.find((view) => view.name === table.dependencies_table[0]);

  const tableMeta = await describeTable(params.databaseName, table.name);
  const tableName = data.tables.find((table) => table.uuid === params.tableId).name;
  const tableData = await getTable(params.databaseName, tableName);

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
            <Tabs defaultValue="fields" className="h-full flex flex-col">
              <TabsList className={cn(tabListStyle, "justify-start")}>
                  <TabsTrigger className={cn(tabTriggerStyle)} value="fields">Fields</TabsTrigger>
                  <TabsTrigger className={cn(tabTriggerStyle)} value="preview">Preview</TabsTrigger>
                  <TabsTrigger className={cn(tabTriggerStyle)} value="query">Query</TabsTrigger>
              </TabsList>
              <Separator />
              <TabsContent value="fields">
                  <FieldsList fields={tableMeta} />
              </TabsContent>
              <TabsContent value="preview">
                  {tableData.length > 0 ? <PreviewTable rows={tableData} /> : <ClickhouseTableRestriction viewId={associated_view.uuid} databaseName={params.databaseName} /> }
              </TabsContent>
              <TabsContent className="flex-grow" value="query">
                {/* add query here */}
                  <div className="p-0 h-full">
                    <QueryInterface table={table} related={data.tables}/>
                  </div>
              </TabsContent>
          </Tabs>
         
        </div>
      </section>  
   
  );
}
