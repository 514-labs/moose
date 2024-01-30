/* eslint-disable turbo/no-undeclared-env-vars */

import { BaseResultSet, createClient } from "@clickhouse/client-web";
import { getCliData } from "app/db";
import { Row, Value, infrastructureMock } from "app/infrastructure/mock";
import { Field } from "app/mock";
import { Card, CardContent } from "components/ui/card";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "components/ui/resizable";
import { Separator } from "components/ui/separator";
import { Table, TableBody, TableCaption, TableCell, TableHead, TableHeader, TableRow } from "components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";


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

interface TableProps {
  rows: Row[];
}

const PreviewTable = ({ rows }: TableProps) => {
  // Get column headers (keys from the first object in the data array)
  const headers = rows.length > 0 ? Object.keys(rows[0]) : [];

  return (
    <Table>
      <TableCaption>A preview of the data in your table.</TableCaption>
      <TableHeader>
        <TableRow>
          {headers.map((header, index) => (
            <TableHead key={index} className="font-medium">
              {header}
            </TableHead>
          ))} 
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.map((row, index) => (
          <TableRow key={index}>
            {headers.map((value, index) => (
              <TableCell key={index}>{row[value]}</TableCell>
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
                                  <div className="text-muted-foreground">{field.type}</div>
                              </div>
                              <span className="flex-grow"/>
                              {/* <div>
                                  {field.rowCount.toLocaleString("en-us")} rows
                              </div> */}
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
    <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-10">
          <div className="text-6xl">{table.name}</div>
          <div className="text-muted-foreground">{table.engine}</div>
        </div>
        <div className="flex flex-row space-x-3 ">
            <Tabs defaultValue="fields" className="flex-grow">
              <TabsList>
                  <TabsTrigger value="fields">Fields</TabsTrigger>
                  <TabsTrigger value="preview">Preview</TabsTrigger>
                  <TabsTrigger value="query">Query</TabsTrigger>
              </TabsList>
              <TabsContent value="fields">
                  <FieldsListCard fields={tableMeta} />
              </TabsContent>
              <TabsContent value="preview">
                <Card>
                  <CardContent className="px-0">
                    <PreviewTable rows={tableData} />
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
