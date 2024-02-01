/* eslint-disable turbo/no-undeclared-env-vars */

import { BaseResultSet, createClient } from "@clickhouse/client-web";
import { getCliData } from "app/db";
import { Field } from "app/mock";
import { PreviewTable } from "components/preview-table";
import QueryInterface from "components/query-interface";
import { tabListStyle, tabTriggerStyle } from "components/style-utils";
import { Card, CardContent } from "components/ui/card";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "components/ui/resizable";
import { Separator } from "components/ui/separator";
import { Table, TableBody, TableCaption, TableCell, TableHead, TableHeader, TableRow } from "components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";
import { Textarea } from "components/ui/textarea";
import { cn } from "lib/utils";


import { unstable_noStore as noStore } from "next/cache";

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

  const resultSet = await client.query({
    query: `SELECT * FROM ${databaseName}.${tableName} LIMIT 50`,
    format: "JSONEachRow",
  });

  return resultSet.json();
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

  const tableMeta = await describeTable(params.databaseName, table.name);
  const tableName = data.tables.find((table) => table.uuid === params.tableId).name;
  const tableData = await getTable(params.databaseName, tableName);

  return (
    <section className="p-4 max-h-screen flex-grow overflow-y-auto flex flex-col">
        <div className="py-10">
          <div className="text-6xl">{table.name}</div>
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
                <Card>
                  <CardContent className="px-0">
                    <PreviewTable rows={tableData} />
                  </CardContent>
                </Card>
                {/* add preview here */}
              </TabsContent>
              <TabsContent className="flex-grow" value="query">
                {/* add query here */}
                  <div className="p-0 h-full">
                    <QueryInterface table={table} />
                  </div>
              </TabsContent>
          </Tabs>
         
        </div>
      </section>  
   
  );
}
