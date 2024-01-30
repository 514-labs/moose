/* eslint-disable turbo/no-undeclared-env-vars */

import { BaseResultSet, createClient } from "@clickhouse/client-web";
import { Row, Value, infrastructureMock } from "app/infrastructure/mock";
import { Field } from "app/mock";
import { Badge, badgeVariants } from "components/ui/badge";
import { Button, buttonVariants } from "components/ui/button";
import { Card, CardContent } from "components/ui/card";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "components/ui/resizable";
import { Separator } from "components/ui/separator";
import { Table, TableBody, TableCaption, TableCell, TableHead, TableHeader, TableRow } from "components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";
import { cn } from "lib/utils";

import { unstable_noStore as noStore } from "next/cache";


async function getTable(databaseId: string, tableId): Promise<any> {
// async function getTable(tableName: string): Promise<any> {
  // const CLICKHOUSE_HOST = process.env.CLICKHOUSE_HOST || "localhost";
  // // Environment variables are always strings
  // const CLICKHOUSE_PORT = process.env.CLICKHOUSE_PORT || "18123";
  // const CLICKHOUSE_USERNAME = process.env.CLICKHOUSE_USERNAME || "panda";
  // const CLICKHOUSE_PASSWORD = process.env.CLICKHOUSE_PASSWORD || "pandapass";
  // const CLICKHOUSE_DB = process.env.CLICKHOUSE_DB || "local";

  // const client = createClient({
  //   host: `http://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
  //   username: CLICKHOUSE_USERNAME,
  //   password: CLICKHOUSE_PASSWORD,
  //   database: CLICKHOUSE_DB,
  // });

  // const resultSet = await client.query({
  //   query: `SELECT * FROM ${tableName} LIMIT 10`,
  //   format: "JSONEachRow",
  // });

  // return resultSet.json();

  try {
    return infrastructureMock.databases.find((db) => db.id === databaseId).tables.find((table) => table.id === tableId);  
  } catch (error) {
    return []
  }
  
}

interface TableProps {
  rows: Row[];
}

const PreviewTable = ({ rows }: TableProps) => {
  // Get column headers (keys from the first object in the data array)
  // const headers = rows.length > 0 ? Object.keys(rows[0]) : [];

  const headers =  rows[0].values.map((value: Value) => value.field);

  console.log(headers);

  return (
    <Table>
      <TableCaption>A preview of the data in your table.</TableCaption>
      <TableHeader>
        <TableRow>
          {headers.map((header, index) => (
            <TableHead key={index} className="font-medium">
              {header.name}
            </TableHead>
          ))} 
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.map((row, index) => (
          <TableRow key={index}>
            {row.values.map((value, index) => (
              <TableCell key={index}>{value.value}</TableCell>
            ))}
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
};


interface FieldsListCardProps {
  fields: Field[]
}


function FieldsListCard({ fields }: FieldsListCardProps) {
  return (
      <Card className="w-full">
          <CardContent className="p-0">
              <ul className="">
                  {fields.map((field, index) => (
                      <li key={index}>
                          <div className="py-2 flex flex-row p-4">
                              <div>
                                  <div className="text-xl">{field.name}</div>
                                  <div className="text-muted-foreground">{field.description}</div>
                              </div>
                              <span className="flex-grow"/>
                              <div>
                                  <Badge className={cn(badgeVariants({ variant: "secondary" }), "mx-4")} key={index}>{field.rowCount.toLocaleString("en-us")} rows</Badge>
                                  <Button className={buttonVariants({ variant: "outline" })}>more</Button>
                              </div>
                          </div>
                          {index < fields.length - 1 && <Separator/>}
                      </li>
                  ))}
              </ul>                
          </CardContent>
      </Card>
  )
}



export default async function Page({
  params,
}: {
  params: {databaseName: string,  tableName: string };
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();

  const view = await getTable(params.databaseName, params.tableName);

  return (
    <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-10">
          <div className="text-6xl">{view.name}</div>
          <div className="text-muted-foreground">{view.description}</div>
        </div>
        <div className="flex flex-row space-x-3 ">
            <Tabs defaultValue="fields" className="flex-grow">
              <TabsList>
                  <TabsTrigger value="fields">Fields</TabsTrigger>
                  <TabsTrigger value="preview">Preview</TabsTrigger>
                  <TabsTrigger value="query">Query</TabsTrigger>
              </TabsList>
              <TabsContent value="fields">
                  <FieldsListCard fields={view.fields} />
              </TabsContent>
              <TabsContent value="preview">
                <Card>
                  <CardContent className="px-0">
                    <PreviewTable rows={view.samples} />
                  </CardContent>
                </Card>
                {/* add preview here */}
              </TabsContent>
              <TabsContent value="query">
                {/* add query here */}
                <Card>
                  <CardContent className="p-0 h-80">
                    <ResizablePanelGroup
                      direction="vertical"
                    >
                      <ResizablePanel defaultSize={80}>
                        <ResizablePanelGroup direction="horizontal">
                          <ResizablePanel defaultSize={75}>
                            <div className="flex h-full items-center justify-center p-6">
                              <span className="font-semibold">Query section</span>
                            </div>
                          </ResizablePanel>
                          <ResizableHandle withHandle />
                          <ResizablePanel defaultSize={25}>
                            <div className="flex h-full items-center justify-center p-6">
                              <span className="font-semibold">Autocomplete objects like fields</span>
                            </div>
                          </ResizablePanel>
                        </ResizablePanelGroup>
                      </ResizablePanel>
                      <ResizableHandle withHandle />
                      <ResizablePanel defaultSize={20}>
                        <div className="flex h-fulltems-center p-6">
                          <span className="font-semibold">Result set</span>
                        </div>
                      </ResizablePanel>
                    </ResizablePanelGroup>
                  </CardContent>
                </Card>
              </TabsContent>
              
          </Tabs>
         
        </div>
      </section>  
   
  );
}
