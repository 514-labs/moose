/* eslint-disable turbo/no-undeclared-env-vars */

import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";

import { unstable_noStore as noStore } from "next/cache";
import Link from "next/link";
import { Separator } from "components/ui/separator";
import { cn, getRelatedInfra } from "lib/utils";
import { tabListStyle, tabTriggerStyle } from "components/style-utils";
import { CliData, DataModel, getCliData } from "app/db";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "components/ui/card";
import { Button } from "components/ui/button";
import CodeCard from "components/code-card";
import SnippetCard from "components/snippet-card";
import { clickhouseJSSnippet, clickhousePythonSnippet, jsSnippet, pythonSnippet } from "lib/snippets";



async function getModel(name: string, data: CliData): Promise<DataModel> {
  try {
    return data.models.find(x => x.name === name);
  } catch (error) {
    return null
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

  const view = infra.tables.find(t => t.name.includes(model.name) && t.engine === "MaterializedView");

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
      <div className="py-10">
        <div className="text-6xl">
          <Link className="text-muted-foreground" href="/"> .. / </Link>
          <Link className="text-muted-foreground" href="/primitives/models"> models </Link>
          <Link href="/primitives/models">/ {model.name} </Link>
        </div>
        <div className="text-muted-foreground py-5 max-w-screen-md">
          Models define the shape of the data that your MooseJS app expects.
          If you want to learn more about them, head to the <a className="underline" href="">documentation</a>
        </div>

        <div className="flex flex-row space-x-3 ">
          <Tabs defaultValue="overview" className="flex-grow">
            <TabsList className={cn(tabListStyle)}>
              <TabsTrigger className={cn(tabTriggerStyle)} value="overview">Overview</TabsTrigger>
              <TabsTrigger className={cn(tabTriggerStyle)} value="usage">Usage</TabsTrigger>
              <TabsTrigger className={cn(tabTriggerStyle)} value="logs">Logs</TabsTrigger>
            </TabsList>
            <TabsContent value="overview">
              <div className=" grid grid-cols-12 gap-4">
                <div className="col-span-12 xl:col-span-6">
                  <Card className="rounded-3xl">
                    <CardHeader className="text-xl text-muted-foreground">Fields</CardHeader>
                    <CardContent >
                        <div >
                          <div className="flex py-4">
                            <div className="grow basis-1">Field Name</div>
                            <div className="grow basis-1"> Type</div>
                            <div className="grow basis-1"> Required?</div>
                            <div className="grow basis-1"> Unique?</div>
                            <div className="grow basis-1"> Primary Key?</div>
                          </div>
                          <Separator />
                        </div>
                      {model.columns.map((field, index) => (
                        <div key={index}>
                          <div className="flex py-4">
                            <div className="grow basis-1 text-muted-foreground">{field.name}</div>
                            <div className="grow basis-1 text-muted-foreground"> {field.data_type}</div>
                            <div className="grow basis-1 text-muted-foreground"> {field.arity}</div>
                            <div className="grow basis-1 text-muted-foreground"> {`${field.unique}`}</div>
                            <div className="grow basis-1 text-muted-foreground"> {`${field.primary_key}`}</div>
                          </div>
                          { index !== model.columns.length - 1 && <Separator />}
                        </div>
                      ))}
                    </CardContent>
                  </Card>
                </div>
                <div className="col-span-12 xl:col-span-6">
                  <Card className="rounded-3xl">
                      <CardHeader className="text-xl text-muted-foreground">Related Infra</CardHeader>
                      <CardContent >
                        {infra && infra.tables.map((table, index) => (
                          <Link key={index} href={`/infrastructure/databases/${table.database}/tables/${table.uuid}`} >
                          <div key={index} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                            <div className="flex flex-row grow">
                              <div key={index} className="flex grow py-4 space-x-4">
                                <div className="grow basis-1">{table.name}</div>
                                <div className="grow basis-1 text-muted-foreground">{table.name.includes("view") ? "view" : "table"}</div>
                              </div>
                            </div>
                            <Separator />
                          </div>
                          </Link>
                        ))}
                        {infra && infra.ingestionPoints.map((ingestionPoint, index) => (
                          <Link key={index} href={`/infrastructure/ingestion-points/${ingestionPoint.route_path.split("/").at(-1)}`} >
                            <div key={index} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                              <div className="flex flex-row grow">
                                <div key={index} className="flex grow py-4 space-x-4">
                                  <div className="grow basis-1">{ingestionPoint.route_path}</div>
                                  <div className="grow basis-1 text-muted-foreground">ingestion point</div>
                                </div>
                              </div>
                              {index === infra.ingestionPoints.length && <Separator />}
                            </div>
                          </Link>
                        ))}
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
                      <CardDescription>When you create a data model, moose automatically spins up infrastructure to ingest data. You can easily push data to this infrastructure in the following ways:</CardDescription>
                    </CardHeader>
                    <CardContent >
                      <div className="pb-4">
                        <h1 className="text-lg">Send data over http to the ingestion point</h1>
                        <SnippetCard title="Ingestion point">
                          <code className="bg-muted">
                            {data.project && `http://${data.project.local_webserver_config.host}:${data.project.local_webserver_config.port}/${infra.ingestionPoints[0].route_path}`}
                          </code>
                        </SnippetCard>
                      </div>
                      <div className="py-4">
                        <h1 className="text-lg">Use an autogenerated SDK</h1>
                        <SnippetCard title="Step 1: Link autogenerated SDKs to make them globally available">
                          <code className="text-nowrap">
                          {`// from the sdk package directory ${data.project && data.project.project_file_location}/.moose/${data.project.name}-sdk`}
                          </code>
                          <code>
                              npm link -g
                          </code>
                        </SnippetCard>
                        <div className="py-4">
                          <SnippetCard title={"Step 2: Link autogenerated sdk to your project from global packages"}>
                              <code className="text-nowrap">
                                {`// your application's directory where your package.json is`}
                              </code>
                              <code>
                                {`npm install ${data.project && data.project.name}-sdk`}
                              </code>
                          </SnippetCard>
                        </div>
                      </div>
                      <div className="py-4">
                        <h1 className="text-lg">Using the language of your choice</h1>
                        <div className="py-4">
                            <CodeCard title="Code" snippets={[
                              {
                                language: "javascript",
                                code: jsSnippet(data, model)
                              },
                              {
                                language: "python",
                                code: pythonSnippet(data, model)
                              }
                            ]} />
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                </div>
                <div className="col-span-12 xl:col-span-6">
                  <Card className="rounded-3xl">
                      <CardHeader className="text-xl  text-muted-foreground">
                        <CardTitle className=" font-normal">Data Out</CardTitle>
                        <CardDescription>When you create a data model, moose automatically spins up infrastructure to ingest data. You can easily extract data from the infrastructure in the following ways:</CardDescription>
                      </CardHeader>
                      <CardContent >
                        <div>
                          <h1 className="text-lg">Exploratory queries</h1>
                          <h2 className="py-2 flex flex-row items-center">
                            <div className="flex flex-col">
                              <span>Query the view directly</span>
                              <span className="text-sm text-muted-foreground">You can run explore your data with sql by querying the view directly</span>
                            </div>
                            <span className="grow" />
                            <Link href={`/infrastructure/databases/${view.database}/tables/${view.uuid}?tab=query`}>
                            <Button variant="outline">go to view</Button>
                            </Link>
                          </h2>
                        </div>
                        <div className="py-8">
                          <h1 className="text-lg">Application Client</h1>
                          <CodeCard title="Clickhouse clients" snippets={[
                              {
                                language: "javascript",
                                code: clickhouseJSSnippet(data, model)
                              },
                              {
                                language: "python",
                                code: clickhousePythonSnippet(data, model)
                              }
                            ]} />
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
          </Tabs>
        </div>
      </div>
    </section>

  );
}
