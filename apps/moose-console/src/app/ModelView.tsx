"use client";

// We will move this eventually
// Views, IngestionPoints and Models have the exact same UI
// the thinking is we will remove the Views and IngestionPoint pages

import { CliData, Table } from "app/db";
import CodeCard from "components/code-card";
import IngestionInstructions from "components/ingestion-instructions";
import ModelTable from "components/model-table";
import QueryInterface from "components/query-interface";
import RelatedInfraTable from "components/related-infra-table";
import { tabListStyle, tabTriggerStyle } from "components/style-utils";
import { Button } from "components/ui/button";
import {
  CardHeader,
  CardContent,
  Card,
  CardTitle,
  CardDescription,
} from "components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";
import {
  cn,
  getModelFromTable,
  getRelatedInfra,
  tableIsQueryable,
} from "lib/utils";
import Link from "next/link";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { useCallback, useState } from "react";

interface TableTabsProps {
  table: Table;
  cliData: CliData;
  jsSnippet: string;
  bashSnippet: string;
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

export default function ModelView({
  table,
  cliData,
  jsSnippet,
  bashSnippet,
  pythonSnippet,
  clickhouseJSSnippet,
  clickhousePythonSnippet,
}: TableTabsProps) {
  const searchParams = useSearchParams();
  const tab = searchParams.get("tab");
  const router = useRouter();
  const pathName = usePathname();

  const [_selectedTab, setSelectedTab] = useState<string>(
    tab ? tab : "overview"
  );
  const model = getModelFromTable(table, cliData);
  const infra = getRelatedInfra(model, cliData, table);
  const associatedView = cliData.tables.find(
    (view) => view.name === table.dependencies_table[0]
  );

  const createTabQueryString = useCallback(
    (tab: string) => {
      const params = new URLSearchParams(searchParams.toString());
      params.set("tab", tab);
      return params.toString();
    },
    [searchParams]
  );

  const ingestionPoint = infra.ingestionPoints[0];

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
        <TabsTrigger className={cn(tabTriggerStyle)} value="setup">
          Setup
        </TabsTrigger>
        <TabsTrigger className={cn(tabTriggerStyle)} value="logs">
          Logs
        </TabsTrigger>
        <TabsTrigger className={cn(tabTriggerStyle)} value="query">
          Query
        </TabsTrigger>
      </TabsList>
      <TabsContent value="overview">
        <div className=" grid grid-cols-12 gap-4">
          <div className="col-span-12 xl:col-span-6">
            <Card className="rounded-3xl">
              <CardHeader>
                <CardTitle>Fields</CardTitle>
              </CardHeader>
              <CardContent>
                <ModelTable datamodel={model} />
              </CardContent>
            </Card>
          </div>
          <div className="col-span-12 xl:col-span-6">
            <Card className="rounded-3xl">
              <CardHeader>
                <CardTitle>Related Infra</CardTitle>
              </CardHeader>
              <CardContent>
                <RelatedInfraTable infra={infra} />
              </CardContent>
            </Card>
          </div>
        </div>
      </TabsContent>
      <TabsContent className="h-full" value="setup">
        <div className=" grid grid-cols-12 gap-4">
          <div className="col-span-12 xl:col-span-6">
            <Card className="rounded-3xl">
              <CardHeader>
                <CardTitle className="font-normal">Data In</CardTitle>
                <CardDescription>
                  When you create a data model, moose automatically spins up
                  infrastructure to ingest data. You can easily push data to
                  this infrastructure in the following ways:
                </CardDescription>
              </CardHeader>
              <CardContent>
                {ingestionPoint && (
                  <IngestionInstructions
                    bashSnippet={bashSnippet}
                    cliData={cliData}
                    jsSnippet={jsSnippet}
                    pythonSnippet={pythonSnippet}
                    ingestionPoint={ingestionPoint}
                  />
                )}
              </CardContent>
            </Card>
          </div>
          <div className="col-span-12 xl:col-span-6">
            <Card className="rounded-3xl">
              <CardHeader>
                <CardTitle>Data Out</CardTitle>
                <CardDescription>
                  When you create a data model, moose automatically spins up
                  infrastructure to extract data. You can easily extract data
                  from the infrastructure in the following ways:
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div>
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
                          `${pathName}?${createTabQueryString("query")}`
                        );
                        setSelectedTab("query");
                      }}
                    >
                      Query
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
      <TabsContent value="logs">
        <Card className="bg-muted rounded-2xl">
          <CardContent className="font-mono p-4">some log content</CardContent>
        </Card>
      </TabsContent>
      <TabsContent className="h-full" value="query">
        {/* add query here */}
        <div className="p-0 h-full">
          {tableIsQueryable(table) && cliData.project ? (
            <QueryInterface
              project={cliData.project}
              table={table}
              related={cliData.tables}
            />
          ) : (
            associatedView && ClickhouseTableRestriction(associatedView)
          )}
        </div>
      </TabsContent>
    </Tabs>
  );
}
