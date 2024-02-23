"use client";

import { CliData, Route, Table } from "app/db";
import CodeCard from "components/code-card";
import IngestionInstructions from "components/ingestion-instructions";
import ModelTable from "components/model-table";
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
import { cn, getModelFromRoute, getRelatedInfra } from "lib/utils";
import Link from "next/link";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { useCallback, useState } from "react";

interface IngestionPointTabsProps {
  ingestionPoint: Route;
  cliData: CliData;
  jsSnippet: string;
  bashSnippet: string;
  pythonSnippet: string;
  clickhouseJSSnippet: string;
  clickhousePythonSnippet: string;
}

function _ClickhouseTableRestriction(view: Table) {
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

export default function IngestionPointTabs({
  ingestionPoint,
  cliData,
  jsSnippet,
  pythonSnippet,
  bashSnippet,
  clickhouseJSSnippet,
  clickhousePythonSnippet,
}: IngestionPointTabsProps) {
  const searchParams = useSearchParams();
  const tab = searchParams.get("tab");
  const router = useRouter();
  const pathName = usePathname();

  const [_, setSelectedTab] = useState<string>(tab ? tab : "overview");
  const model = getModelFromRoute(ingestionPoint, cliData);
  const infra = getRelatedInfra(model, cliData, ingestionPoint);

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
        <TabsTrigger className={cn(tabTriggerStyle)} value="logs">
          Logs
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
      <TabsContent className="h-full" value="usage">
        <div className=" grid grid-cols-12 gap-4">
          <div className="col-span-12 xl:col-span-6">
            <Card className="rounded-3xl">
              <CardHeader>
                <CardTitle>Data In</CardTitle>
                <CardDescription>
                  When you create a data model, moose automatically spins up
                  infrastructure to ingest data. You can easily push data to
                  this infrastructure in the following ways:
                </CardDescription>
              </CardHeader>
              <CardContent>
                <IngestionInstructions
                  bashSnippet={bashSnippet}
                  cliData={cliData}
                  ingestionPoint={ingestionPoint}
                  jsSnippet={jsSnippet}
                  pythonSnippet={pythonSnippet}
                />
              </CardContent>
            </Card>
          </div>
          <div className="col-span-12 xl:col-span-6">
            <Card className="rounded-3xl">
              <CardHeader>
                <CardTitle>Data Out</CardTitle>
                <CardDescription>
                  When you create a data model, moose automatically spins up
                  infrastructure to ingest data. You can easily extract data
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
      <TabsContent className="h-full" value="logs">
        {/* add query here */}
        <div className="p-0 h-full">
          <code>some logs</code>
        </div>
      </TabsContent>
    </Tabs>
  );
}
