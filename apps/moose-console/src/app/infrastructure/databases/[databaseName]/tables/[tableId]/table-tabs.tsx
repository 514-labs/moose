"use client";

import { CliData, Table } from "app/db";
import CodeCard from "components/code-card";
import QueryInterface from "components/query-interface";
import SnippetCard from "components/snippet-card";
import { tabListStyle, tabTriggerStyle } from "components/style-utils";
import { Button } from "components/ui/button";
import {
  CardHeader,
  CardContent,
  Card,
  CardTitle,
  CardDescription,
} from "components/ui/card";
import { Separator } from "components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";
import { cn, getModelFromTable, getRelatedInfra, tableIsView } from "lib/utils";
import Link from "next/link";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { useCallback, useState } from "react";

interface TableTabsProps {
  table: Table;
  cliData: CliData;
  jsSnippet: string;
  pythonSnippet: string;
  clickhouseJSSnippet: string;
  clickhousePythonSnippet: string;
}

function ClickhouseTableRestriction(view: Table) {
  return (
    <div className="py-4">
      <div className="text-muted-foreground max-w-xl">
        This table is an ingestion Clickhouse table. You cannnot query it
        directly. To get a preview of the data, head to its associated view
      </div>
      <Link
        className=" underline"
        href={`/infrastructure/databases/${view.database}/tables/${view.uuid}?tab=query`}
      >
        <Button variant="default" className="mt-4">
          go to view
        </Button>
      </Link>
    </div>
  );
}

export default function TableTabs({
  table,
  cliData,
  jsSnippet,
  pythonSnippet,
  clickhouseJSSnippet,
  clickhousePythonSnippet,
}: TableTabsProps) {
  const searchParams = useSearchParams();
  const tab = searchParams.get("tab");
  const router = useRouter();
  const pathName = usePathname();

  const [_selectedTab, setSelectedTab] = useState<string>(
    tab ? tab : "overview",
  );
  const model = getModelFromTable(table, cliData);
  const infra = getRelatedInfra(model, cliData, table);
  const associated_view = cliData.tables.find(
    (view) => view.name === table.dependencies_table[0],
  );

  const createTabQueryString = useCallback(
    (tab: string) => {
      const params = new URLSearchParams(searchParams.toString());
      params.set("tab", tab);
      return params.toString();
    },
    [searchParams],
  );

  return (
    <Tabs
      value={tab ? tab : "overview"}
      className="h-full"
      onValueChange={(value) => {
        router.push(`${pathName}?${createTabQueryString(value)}`);
        setSelectedTab(value);
      }}
    >
      <TabsList className={cn(tabListStyle, "flex-grow-0")}>
        <TabsTrigger className={cn(tabTriggerStyle)} value="overview">
          Overview
        </TabsTrigger>
        <TabsTrigger className={cn(tabTriggerStyle)} value="usage">
          Usage
        </TabsTrigger>
        <TabsTrigger className={cn(tabTriggerStyle)} value="query">
          Query
        </TabsTrigger>
      </TabsList>
      <TabsContent value="overview">
        <div className=" grid grid-cols-12 gap-4">
          <div className="col-span-12 xl:col-span-6">
            <Card className="rounded-3xl">
              <CardHeader className="text-xl text-muted-foreground">
                Fields
              </CardHeader>
              <CardContent>
                <div>
                  <div className="flex py-4">
                    <div className="grow basis-1">Field Name</div>
                    <div className="grow basis-1"> Type</div>
                    <div className="grow basis-1"> Required?</div>
                    <div className="grow basis-1"> Unique?</div>
                    <div className="grow basis-1"> Primary Key?</div>
                  </div>
                  <Separator />
                </div>
                {model &&
                  model.columns.map((field, index) => (
                    <div key={index}>
                      <div className="flex py-4">
                        <div className="grow basis-1 text-muted-foreground">
                          {field.name}
                        </div>
                        <div className="grow basis-1 text-muted-foreground">
                          {" "}
                          {field.data_type}
                        </div>
                        <div className="grow basis-1 text-muted-foreground">
                          {" "}
                          {field.arity}
                        </div>
                        <div className="grow basis-1 text-muted-foreground">
                          {" "}
                          {`${field.unique}`}
                        </div>
                        <div className="grow basis-1 text-muted-foreground">
                          {" "}
                          {`${field.primary_key}`}
                        </div>
                      </div>
                      {index !== model.columns.length - 1 && <Separator />}
                    </div>
                  ))}
              </CardContent>
            </Card>
          </div>
          <div className="col-span-12 xl:col-span-6">
            <Card className="rounded-3xl">
              <CardHeader className="text-xl text-muted-foreground">
                Related Infra
              </CardHeader>
              <CardContent>
                {infra &&
                  infra.tables.map((table, index) => (
                    <Link
                      key={index}
                      href={`/infrastructure/databases/${table.database}/tables/${table.uuid}`}
                    >
                      <div
                        key={index}
                        className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer"
                      >
                        <div className="flex flex-row grow">
                          <div key={index} className="flex grow py-4 space-x-4">
                            <div className="grow basis-1">{table.name}</div>
                            <div className="grow basis-1 text-muted-foreground">
                              {table.name.includes("view") ? "view" : "table"}
                            </div>
                          </div>
                        </div>
                        <Separator />
                      </div>
                    </Link>
                  ))}
                {infra &&
                  infra.ingestionPoints.map((ingestionPoint, index) => (
                    <Link
                      key={index}
                      href={`/infrastructure/ingestion-points/${ingestionPoint.route_path.split("/").at(-1)}`}
                    >
                      <div
                        key={index}
                        className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer"
                      >
                        <div className="flex flex-row grow">
                          <div key={index} className="flex grow py-4 space-x-4">
                            <div className="grow basis-1">
                              {ingestionPoint.route_path}
                            </div>
                            <div className="grow basis-1 text-muted-foreground">
                              ingestion point
                            </div>
                          </div>
                        </div>
                        {index === infra.ingestionPoints.length && (
                          <Separator />
                        )}
                      </div>
                    </Link>
                  ))}
              </CardContent>
            </Card>
          </div>
        </div>
      </TabsContent>
      <TabsContent className="h-full" value="usage">
        <div className=" grid grid-cols-12 gap-4">
          <div className="col-span-12 xl:col-span-6">
            <Card className="rounded-3xl">
              <CardHeader className="text-xl  text-muted-foreground">
                <CardTitle className=" font-normal">Data In</CardTitle>
                <CardDescription>
                  When you create a data model, moose automatically spins up
                  infrastructure to ingest data. You can easily push data to
                  this infrastructure in the following ways:
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="pb-4">
                  <h1 className="text-lg">
                    Send data over http to the ingestion point
                  </h1>
                  <SnippetCard title="Ingestion point">
                    <code className="bg-muted">
                      {cliData.project &&
                        `http://${cliData.project.local_webserver_config.host}:${cliData.project.local_webserver_config.port}/${infra.ingestionPoints[0].route_path}`}
                    </code>
                  </SnippetCard>
                </div>
                <div className="py-4">
                  <h1 className="text-lg">Use an autogenerated SDK</h1>
                  <SnippetCard title="Step 1: Link autogenerated SDKs to make them globally available">
                    <code className="text-nowrap">
                      {`// from the sdk package directory ${cliData.project && cliData.project.project_file_location}/.moose/${cliData.project.name}-sdk`}
                    </code>
                    <code>npm link -g</code>
                  </SnippetCard>
                  <div className="py-4">
                    <SnippetCard
                      title={
                        "Step 2: Link autogenerated sdk to your project from global packages"
                      }
                    >
                      <code className="text-nowrap">
                        {`// your application's directory where your package.json is`}
                      </code>
                      <code>
                        {`npm install ${cliData.project && cliData.project.name}-sdk`}
                      </code>
                    </SnippetCard>
                  </div>
                </div>
                <div className="py-4">
                  <h1 className="text-lg">Using the language of your choice</h1>
                  <div className="py-4">
                    <CodeCard
                      title="Code"
                      snippets={[
                        {
                          language: "javascript",
                          code: jsSnippet,
                        },
                        {
                          language: "python",
                          code: pythonSnippet,
                        },
                      ]}
                    />
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
          <div className="col-span-12 xl:col-span-6">
            <Card className="rounded-3xl">
              <CardHeader className="text-xl  text-muted-foreground">
                <CardTitle className=" font-normal">Data Out</CardTitle>
                <CardDescription>
                  When you create a data model, moose automatically spins up
                  infrastructure to ingest data. You can easily extract data
                  from the infrastructure in the following ways:
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div>
                  <h1 className="text-lg">Exploratory queries</h1>
                  <h2 className="py-2 flex flex-row items-center">
                    <div className="flex flex-col">
                      <span>Query the view directly</span>
                      <span className="text-sm text-muted-foreground">
                        You can run explore your data with sql by querying the
                        view directly
                      </span>
                    </div>
                    <span className="grow" />

                    <Button
                      variant="outline"
                      onClick={() => {
                        router.push(
                          `${pathName}?${createTabQueryString("query")}`,
                        );
                        setSelectedTab("query");
                      }}
                    >
                      go to view
                    </Button>
                  </h2>
                </div>
                <div className="py-8">
                  <h1 className="text-lg">Application Client</h1>
                  <CodeCard
                    title="Clickhouse clients"
                    snippets={[
                      {
                        language: "javascript",
                        code: clickhouseJSSnippet,
                      },
                      {
                        language: "python",
                        code: clickhousePythonSnippet,
                      },
                    ]}
                  />
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </TabsContent>
      <TabsContent className="h-full" value="query">
        {/* add query here */}
        <div className="p-0 h-full">
          {tableIsView(table) ? (
            <QueryInterface table={table} related={cliData.tables} />
          ) : (
            ClickhouseTableRestriction(associated_view)
          )}
        </div>
      </TabsContent>
    </Tabs>
  );
}
