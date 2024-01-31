/* eslint-disable turbo/no-undeclared-env-vars */

import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";

import { unstable_noStore as noStore } from "next/cache";
import { Model, modelMock } from "../mock";
import { infrastructureMock } from "app/infrastructure/mock";
import Link from "next/link";
import { Separator } from "components/ui/separator";
import { cn } from "lib/utils";
import { tabListStyle, tabTriggerStyle } from "components/style-utils";


async function getModel(modelId: string): Promise<Model> {
  try {
    return modelMock.models.find((q) => q.id === modelId);
  } catch (error) {
    return null
  }
}



export default async function Page({
  params,
}: {
  params: { modelId: string };
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();

  const model = await getModel(params.modelId);

  const infra = infrastructureMock;

  return (
    <section className="p-4 max-h-screen overflow-y-auto">
      <div className="py-10">
        <div className="text-6xl">
          <Link className="text-muted-foreground" href="/primitives"> ... / </Link>
          <Link className="text-muted-foreground" href="/primitives/models"> Models </Link>
          <Link href="/primitives/models">/ {model.name} </Link>
        </div>
        <div className="text-muted-foreground py-5 max-w-screen-md">
          Models define the shape of the data that your MooseJS app expects.
          If you want to learn more about them, head to the <a className="underline" href="">documentation</a>
        </div>

        <div className="flex flex-row space-x-3 ">
          <Tabs defaultValue="ingestionPoints" className="flex-grow">
            <TabsList className={cn(tabListStyle)}>
              <TabsTrigger className={cn(tabTriggerStyle)} value="ingestionPoints">Ingestion Points</TabsTrigger>
              <TabsTrigger className={cn(tabTriggerStyle)} value="queues">Queues</TabsTrigger>
              <TabsTrigger className={cn(tabTriggerStyle)} value="tables">Tables</TabsTrigger>
              <TabsTrigger className={cn(tabTriggerStyle)} value="views">Views</TabsTrigger>
              <TabsTrigger className={cn(tabTriggerStyle)} value="snippets">Snippets</TabsTrigger>
            </TabsList>
            <Separator />
            <TabsContent value="ingestionPoints">
              {infra.ingestionPoints.map((ingestionPoint, index) => (
                <Link key={index} href={`/infrastructure/ingestion-points/${ingestionPoint.id}`} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                    <div className="py-4 text-muted-foreground">{ingestionPoint.name}</div>
                    <Separator />
                  </div>
                </Link>
              ))}
            </TabsContent>
            <TabsContent value="queues">
              {infra.ingestionPoints.map((queue, index) => (
                <Link key={index} href={`/infrastructure/queues/${queue.id}`} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                    <div className="py-4 text-muted-foreground">{queue.name}</div>
                    <Separator />
                  </div>
                </Link>
              ))}
            </TabsContent>
            <TabsContent value="tables">
              {infra.databases.flatMap(x => x.tables.map((t) => ({ databaseId: x.id, ...t }))).map((table, index) => (
                <Link key={index} href={`/infrastructure/databases/${table.databaseId}/tables/${table.id}`} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                    <div className="py-4 text-muted-foreground">{table.name}</div>
                    <Separator />
                  </div>
                </Link>
              ))}
            </TabsContent>
            <TabsContent value="views">
              {infra.databases.flatMap(x => x.views.map((t) => ({ databaseId: x.id, ...t }))).map((view, index) => (
                <Link key={index} href={`/infrastructure/databases/${view.databaseId}/views/${view.id}`} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                    <div className="py-4 text-muted-foreground">{view.name}</div>
                    <Separator />
                  </div>
                </Link>
              ))}

            </TabsContent>
            <TabsContent value="snippets">
              <code className="font-mono">Some code</code>
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </section>

  );
}
