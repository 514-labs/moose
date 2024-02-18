/* eslint-disable turbo/no-undeclared-env-vars */

import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";

import { unstable_noStore as noStore } from "next/cache";
import Link from "next/link";
import { cn, getRelatedInfra } from "lib/utils";
import { tabListStyle, tabTriggerStyle } from "components/style-utils";
import { CliData, DataModel, getCliData } from "app/db";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "components/ui/card";
import { Button } from "components/ui/button";
import CodeCard from "components/code-card";
import {
  bashSnippet,
  clickhouseJSSnippet,
  clickhousePythonSnippet,
  jsSnippet,
  pythonSnippet,
} from "lib/snippets";
import ModelTable from "components/model-table";
import IngestionInstructions from "components/ingestion-instructions";
import RelatedInfraTable from "components/related-infra-table";

async function getModel(name: string, data: CliData): Promise<DataModel> {
  try {
    return data.models.find((x) => x.name === name);
  } catch (error) {
    return null;
  }
}

export default async function Page({
  params,
}: {
  params: { modelName: string };
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();

  const data = await getCliData();
  const model = await getModel(params.modelName, data);
  const infra = getRelatedInfra(model, data, model);

  const view = infra.tables.find(
    (t) => t.name.includes(model.name) && t.engine === "MaterializedView",
  );

  const jsCodeSnippet = jsSnippet(data, model);
  const pythonCodeSnippet = pythonSnippet(data, model);
  const bashCodeSnippet = bashSnippet(data, model);

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
      <div className="py-10">
        <div className="text-6xl">
          <Link className="text-muted-foreground" href="/">
            {" "}
            .. /{" "}
          </Link>
          <Link className="text-muted-foreground" href="/primitives/models">
            {" "}
            models{" "}
          </Link>
          <Link href="/primitives/models">/ {model.name} </Link>
        </div>
      </div>

      <div className="flex flex-row space-x-3 ">
        <Tabs defaultValue="overview" className="flex-grow">
          <TabsList className={cn(tabListStyle)}>
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
                  <CardHeader className="text-xl text-muted-foreground">
                    Fields
                  </CardHeader>
                  <CardContent>
                    <ModelTable datamodel={model} />
                  </CardContent>
                </Card>
              </div>
              <div className="col-span-12 xl:col-span-6">
                <Card className="rounded-3xl">
                  <CardHeader className="text-xl text-muted-foreground">
                    Related Infra
                  </CardHeader>
                  <CardContent>
                    <RelatedInfraTable infra={infra} />
                  </CardContent>
                </Card>
              </div>
            </div>
          </TabsContent>
          <TabsContent value="usage">
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
                    <IngestionInstructions
                      bashSnippet={bashCodeSnippet}
                      pythonSnippet={pythonCodeSnippet}
                      jsSnippet={jsCodeSnippet}
                      cliData={data}
                      ingestionPoint={infra.ingestionPoints[0]}
                    />
                  </CardContent>
                </Card>
              </div>
              <div className="col-span-12 xl:col-span-6">
                <Card className="rounded-3xl">
                  <CardHeader className="text-xl  text-muted-foreground">
                    <CardTitle className=" font-normal">Data Out</CardTitle>
                    <CardDescription>
                      When you create a data model, moose automatically
                      provisions your database with a ingestion table and view.
                      You can easily extract data from the view in the following
                      ways:
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div>
                      <h1 className="text-lg">Exploratory queries</h1>
                      <h2 className="py-2 flex flex-row items-center">
                        <div className="flex flex-col">
                          <span>Query the view directly</span>
                          <span className="text-sm text-muted-foreground">
                            You can run explore your data with sql by querying
                            the view directly
                          </span>
                        </div>
                        <span className="grow" />
                        <Link
                          href={`/infrastructure/databases/${view.database}/tables/${view.uuid}?tab=query`}
                        >
                          <Button variant="outline">go to view</Button>
                        </Link>
                      </h2>
                    </div>
                    <div className="py-8">
                      <h1 className="text-lg">Application Client</h1>
                      <CodeCard
                        title="Clickhouse clients"
                        snippets={[
                          {
                            language: "javascript",
                            code: clickhouseJSSnippet(data, model),
                          },
                          {
                            language: "python",
                            code: clickhousePythonSnippet(data, model),
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
              <CardContent className="font-mono p-4">
                some log content
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </section>
  );
}
